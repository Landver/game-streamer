use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    sync::{Mutex, RwLock},
};
use tracing::{error, info, warn};
use webrtc::{
    api::{media_engine::MediaEngine, APIBuilder},
    data_channel::data_channel_message::DataChannelMessage,
    ice_transport::ice_candidate::RTCIceCandidateInit,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::track_local_static_sample::TrackLocalStaticSample,
};

use crate::{input_injector, models::SignalMessage, state::AppState};

const BOT_PEER_ID: &str = "ffmpeg-bot";

type SessionPeerKey = String;

struct StreamSession {
    peer_connection: Arc<RTCPeerConnection>,
    ffmpeg_child: Mutex<Child>,
}

#[derive(Default)]
pub struct MediaBridge {
    sessions: Arc<RwLock<HashMap<SessionPeerKey, Arc<StreamSession>>>>,
}

impl MediaBridge {
    pub fn is_bot_target(target_peer: &str) -> bool {
        target_peer == BOT_PEER_ID
    }

    pub async fn handle_offer(
        &self,
        state: AppState,
        session_id: String,
        from_peer: String,
        offer_sdp: String,
    ) -> Result<(), String> {
        let mut media_engine = MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|err| format!("register_default_codecs failed: {err}"))?;
        let api = APIBuilder::new().with_media_engine(media_engine).build();
        let peer_connection = Arc::new(
            api.new_peer_connection(RTCConfiguration::default())
                .await
                .map_err(|err| format!("new_peer_connection failed: {err}"))?,
        );

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: "video/H264".to_owned(),
                clock_rate: 90_000,
                channels: 0,
                sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
                    .to_owned(),
                rtcp_feedback: vec![],
            },
            "video".to_owned(),
            "ffmpeg".to_owned(),
        ));

        let sender = peer_connection
            .add_track(video_track.clone())
            .await
            .map_err(|err| format!("add_track failed: {err}"))?;

        peer_connection.on_data_channel(Box::new(move |dc| {
            Box::pin(async move {
                if dc.label() != "input" {
                    return;
                }
                dc.on_open(Box::new(|| {
                    Box::pin(async move {
                        info!("input_channel_open");
                    })
                }));
                dc.on_message(Box::new(move |msg: DataChannelMessage| {
                    Box::pin(async move {
                        let Ok(text) = String::from_utf8(msg.data.to_vec()) else {
                            warn!("input_event_ignored invalid_utf8");
                            return;
                        };
                        match input_injector::inject_from_json(&text) {
                            Ok(()) => info!("input_event_received"),
                            Err(err) => warn!("input_event_failed error={err}"),
                        }
                    })
                }));
            })
        }));

        tokio::spawn(async move {
            let mut rtcp = vec![0_u8; 1500];
            while sender.read(&mut rtcp).await.is_ok() {}
        });

        let state_for_ice = state.clone();
        let session_for_ice = session_id.clone();
        let from_for_ice = from_peer.clone();
        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            let state_for_ice = state_for_ice.clone();
            let session_for_ice = session_for_ice.clone();
            let from_for_ice = from_for_ice.clone();
            Box::pin(async move {
                let Some(candidate) = candidate else {
                    return;
                };
                let Ok(json) = candidate.to_json() else {
                    return;
                };
                let Ok(payload) = serde_json::to_string(&json) else {
                    return;
                };
                enqueue_message(
                    &state_for_ice,
                    &session_for_ice,
                    &from_for_ice,
                    SignalMessage::IceCandidate {
                        from: BOT_PEER_ID.to_owned(),
                        to: from_for_ice.clone(),
                        candidate: payload,
                    },
                )
                .await;
            })
        }));

        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            Box::pin(async move {
                info!("ffmpeg_bot peer_connection_state={state:?}");
                if state == RTCPeerConnectionState::Failed {
                    warn!("ffmpeg_bot peer connection failed");
                }
            })
        }));

        peer_connection
            .set_remote_description(
                RTCSessionDescription::offer(offer_sdp)
                    .map_err(|err| format!("offer sdp parse failed: {err}"))?,
            )
            .await
            .map_err(|err| format!("set_remote_description failed: {err}"))?;

        let answer = peer_connection
            .create_answer(None)
            .await
            .map_err(|err| format!("create_answer failed: {err}"))?;
        peer_connection
            .set_local_description(answer.clone())
            .await
            .map_err(|err| format!("set_local_description failed: {err}"))?;

        enqueue_message(
            &state,
            &session_id,
            &from_peer,
            SignalMessage::Answer {
                from: BOT_PEER_ID.to_owned(),
                to: from_peer.clone(),
                sdp: answer.sdp,
            },
        )
        .await;

        let ffmpeg_child = spawn_ffmpeg_process().await?;
        let session_key = session_peer_key(&session_id, &from_peer);
        let stream_session = Arc::new(StreamSession {
            peer_connection: peer_connection.clone(),
            ffmpeg_child: Mutex::new(ffmpeg_child),
        });
        self.sessions
            .write()
            .await
            .insert(session_key.clone(), stream_session.clone());

        tokio::spawn(async move {
            if let Err(err) = pump_h264_to_track(stream_session.clone(), video_track).await {
                error!("ffmpeg_bot stream failed key={session_key} error={err}");
            }
            let _ = stream_session.peer_connection.close().await;
        });

        info!("ffmpeg_spawned session={session_id} to_peer={from_peer}");
        Ok(())
    }

    pub async fn handle_remote_ice(
        &self,
        session_id: &str,
        from_peer: &str,
        candidate_json: &str,
    ) -> Result<(), String> {
        let key = session_peer_key(session_id, from_peer);
        let sessions = self.sessions.read().await;
        let Some(stream_session) = sessions.get(&key) else {
            return Err("bot peer connection not found".to_owned());
        };
        let candidate: RTCIceCandidateInit = serde_json::from_str(candidate_json)
            .map_err(|err| format!("parse remote ice failed: {err}"))?;
        stream_session
            .peer_connection
            .add_ice_candidate(candidate)
            .await
            .map_err(|err| format!("add_ice_candidate failed: {err}"))?;
        Ok(())
    }
}

fn session_peer_key(session_id: &str, peer_id: &str) -> String {
    format!("{session_id}:{peer_id}")
}

async fn spawn_ffmpeg_process() -> Result<Child, String> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-loglevel")
        .arg("warning")
        .arg("-init_hw_device")
        .arg("d3d11va=dx")
        .arg("-init_hw_device")
        .arg("qsv=qs@dx")
        .arg("-filter_hw_device")
        .arg("dx")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg("ddagrab=framerate=60:output_idx=0:draw_mouse=1")
        .arg("-vf")
        .arg("hwmap=derive_device=qsv,format=qsv")
        .arg("-an")
        .arg("-c:v")
        .arg("h264_qsv")
        .arg("-profile:v")
        .arg("baseline")
        .arg("-preset")
        .arg("veryfast")
        .arg("-g")
        .arg("60")
        .arg("-keyint_min")
        .arg("60")
        .arg("-b:v")
        .arg("5M")
        .arg("-maxrate")
        .arg("5M")
        .arg("-bufsize")
        .arg("5M")
        .arg("-bf")
        .arg("0")
        .arg("-look_ahead")
        .arg("0")
        .arg("-async_depth")
        .arg("1")
        .arg("-bsf:v")
        .arg("h264_metadata=aud=insert")
        .arg("-f")
        .arg("h264")
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    cmd.spawn()
        .map_err(|err| format!("ffmpeg spawn failed: {err}"))
}

async fn pump_h264_to_track(
    stream_session: Arc<StreamSession>,
    video_track: Arc<TrackLocalStaticSample>,
) -> Result<(), String> {
    let mut child = stream_session.ffmpeg_child.lock().await;
    let Some(mut stdout) = child.stdout.take() else {
        return Err("ffmpeg stdout not piped".to_owned());
    };

    let mut buf = [0_u8; 8192];
    let mut annexb = Vec::<u8>::new();
    let mut sent_samples: u64 = 0;
    let mut saw_first = false;
    let mut current_access_unit = Vec::<u8>::new();

    loop {
        let read = stdout
            .read(&mut buf)
            .await
            .map_err(|err| format!("ffmpeg stdout read failed: {err}"))?;
        if read == 0 {
            break;
        }
        annexb.extend_from_slice(&buf[..read]);
        while let Some(nal_unit) = pop_annexb_nal_unit(&mut annexb) {
            let nal_type = annexb_nal_type(&nal_unit);
            if nal_type == Some(9) && !current_access_unit.is_empty() {
                write_h264_sample(&video_track, &current_access_unit).await?;
                current_access_unit.clear();
                sent_samples += 1;
                if !saw_first {
                    saw_first = true;
                    info!("first_frame_ingested");
                }
                if sent_samples % 120 == 0 {
                    info!("track_active samples_sent={sent_samples}");
                }
            }
            current_access_unit.extend_from_slice(&nal_unit);
        }
    }

    if !current_access_unit.is_empty() {
        write_h264_sample(&video_track, &current_access_unit).await?;
    }

    let _ = child.kill().await;
    Ok(())
}

async fn write_h264_sample(
    video_track: &Arc<TrackLocalStaticSample>,
    sample_bytes: &[u8],
) -> Result<(), String> {
    video_track
        .write_sample(&Sample {
            data: sample_bytes.to_vec().into(),
            duration: Duration::from_millis(33),
            ..Default::default()
        })
        .await
        .map_err(|err| format!("write_sample failed: {err}"))
}

fn annexb_nal_type(nal_unit: &[u8]) -> Option<u8> {
    let start_code_len = if nal_unit.len() >= 4
        && nal_unit[0] == 0
        && nal_unit[1] == 0
        && nal_unit[2] == 0
        && nal_unit[3] == 1
    {
        4
    } else if nal_unit.len() >= 3 && nal_unit[0] == 0 && nal_unit[1] == 0 && nal_unit[2] == 1 {
        3
    } else {
        return None;
    };
    nal_unit.get(start_code_len).map(|byte| byte & 0x1F)
}

fn pop_annexb_nal_unit(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let first = find_start_code(buffer, 0)?;
    let second = find_start_code(buffer, first + 3)?;
    if first > 0 {
        buffer.drain(0..first);
        let second_shifted = second - first;
        let sample = buffer[..second_shifted].to_vec();
        buffer.drain(0..second_shifted);
        return Some(sample);
    }
    let sample = buffer[..second].to_vec();
    buffer.drain(0..second);
    Some(sample)
}

fn find_start_code(buffer: &[u8], from: usize) -> Option<usize> {
    if buffer.len() < 4 || from >= buffer.len().saturating_sub(3) {
        return None;
    }
    let mut i = from;
    while i + 3 < buffer.len() {
        if buffer[i] == 0 && buffer[i + 1] == 0 && buffer[i + 2] == 1 {
            return Some(i);
        }
        if i + 4 < buffer.len()
            && buffer[i] == 0
            && buffer[i + 1] == 0
            && buffer[i + 2] == 0
            && buffer[i + 3] == 1
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

async fn enqueue_message(state: &AppState, session_id: &str, to_peer: &str, msg: SignalMessage) {
    let mut sessions = state.sessions.write().await;
    let Some(session) = sessions.get_mut(session_id) else {
        return;
    };
    let Some(inbox) = session.inboxes.get_mut(to_peer) else {
        return;
    };
    inbox.push_back(msg);
}
