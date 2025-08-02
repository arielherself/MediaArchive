use std::path::Path;
use std::sync::LazyLock;
use std::{collections::HashMap, convert::Infallible, fs::File, io::Read, sync::Arc};

use qbit_rs::model::{AddTorrentArg, Credential, TorrentFile, TorrentSource};
use qbit_rs::Qbit;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use warp::Filter;
use toml::Table;

#[derive(Serialize)]
struct LocalFile {
    name: String,
    full_path: String,
}

#[derive(Serialize)]
struct LocalFileList {
    files: Vec<LocalFile>,
}

#[derive(Serialize)]
struct Torrent {
    name: String,
    progress: f64,
    size: i64,
    dlspeed: i64,
    upspeed: i64,
    eta: i64,
    content_path: String,
    hash: String,
    status: i64,
}

#[derive(Serialize)]
struct TorrentList {
    torrents: Vec<Torrent>,
}

#[derive(Serialize, Deserialize)]
struct ResponseEntry {
    id: usize,
    promotion_time_type: i64,
    promotion_until: String,
    leechers: usize,
    seeders: usize,
    name: String,
    small_descr: String,
    times_completed: usize,
    size: usize,
    added: String,
    hr: usize,
    info_hash: String,
    downhash: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
    status: i64,
    data: Vec<ResponseEntry>,
}

#[tokio::main]
async fn main() {
    static CONFIG: LazyLock<toml::map::Map<String, toml::Value>> = LazyLock::new(|| {
        let mut file = File::open("config.toml").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        content.parse::<Table>().unwrap()
    });

    let qb = Arc::new(Mutex::new({
        let qb_url = CONFIG.get("qb-url").unwrap().as_str().unwrap();
        let qb_username = CONFIG.get("qb-username").unwrap().as_str().unwrap();
        let qb_password = CONFIG.get("qb-password").unwrap().as_str().unwrap();
        let credential = Credential::new(qb_username, qb_password);
        Qbit::new(qb_url, credential)
    }));

    let correct_token = CONFIG.get("access-token").unwrap().as_str().unwrap();

    let client = Arc::new(Mutex::new(reqwest::Client::new()));

    let ping = warp::get().and(warp::path!("ping")).map(|| "pong!");

    let search = warp::get()
        .and(warp::path!("search"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let client = Arc::clone(&client);
            move |query: HashMap<String, String>| {
                let client = Arc::clone(&client);
                async move {
                    let token = query.get("token").unwrap().to_owned();
                    if token != correct_token {
                        return Ok("".to_string());
                    }
                    let keyword = query.get("keyword").unwrap().to_owned();

                    let mut params = HashMap::new();
                    params.insert("keyword", keyword);
                    params.insert("page_size", "100".to_string());
                    let response = client.lock().await
                        .post("https://api.hddolby.com/api/v1/torrent/search")
                        .header("x-api-key", CONFIG.get("passkey").unwrap().as_str().unwrap().to_string())
                        .form(&params)
                        .send().await.unwrap();

                    let response_text = response.text().await.unwrap();

                    let response_data = serde_json::from_str::<Response>(&response_text).unwrap();

                    Ok::<String, Infallible>(serde_json::to_string(&response_data).unwrap())
                }
            }
        });

    let torrent_list = warp::get()
        .and(warp::path!("torrent-list"))
        .and_then({
            let qb = Arc::clone(&qb);
            move || {
                let qb = Arc::clone(&qb);
                async move {
                    let qb_torrent_list = qb.lock().await.get_torrent_list(Default::default()).await.unwrap();
                    let save_root = CONFIG.get("save-root").unwrap().as_str().unwrap().to_string();
                    let torrent_list = TorrentList {
                        torrents: qb_torrent_list.into_iter().map(|tr| {
                            let full_path = Path::new(tr.content_path.as_ref().unwrap());
                            let rel_path = Path::new(full_path).strip_prefix(&save_root).unwrap_or(Path::new(""));
                            Torrent {
                                name: tr.name.unwrap(),
                                progress: tr.progress.unwrap(),
                                size: tr.size.unwrap(),
                                dlspeed: tr.dlspeed.unwrap(),
                                upspeed: tr.upspeed.unwrap(),
                                eta: tr.eta.unwrap(),
                                content_path: rel_path.to_string_lossy().to_string(),
                                hash: tr.hash.unwrap(),
                                status: match tr.state.unwrap() {
                                    qbit_rs::model::State::Uploading | qbit_rs::model::State::StalledUP => 0,
                                    qbit_rs::model::State::Downloading | qbit_rs::model::State::CheckingUP | qbit_rs::model::State::CheckingDL | qbit_rs::model::State::Allocating | qbit_rs::model::State::MetaDL | qbit_rs::model::State::StalledDL => 1,
                                    _ => 2,
                                }
                            }
                        }).collect()
                    };
                    Ok::<String, Infallible>(serde_json::to_string(&torrent_list).unwrap())
                }
            }
        });

    let download = warp::get()
        .and(warp::path!("download"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let client = Arc::clone(&client);
            let qb = Arc::clone(&qb);
            move |query: HashMap<String, String>| {
                let client = Arc::clone(&client);
                let qb = Arc::clone(&qb);
                async move {
                    let token = query.get("token").unwrap().to_owned();
                    if token != correct_token {
                        return Ok("".to_string());
                    }
                    let id = query.get("id").unwrap().to_owned();
                    let downhash = query.get("downhash").unwrap().to_owned();
                    let response = client.lock().await
                        .get(format!("https://www.hddolby.com/download.php?id={id}&downhash={downhash}"))
                        .header("User-Agent", "MediaArchive/0.1.0")
                        .send().await.unwrap();

                    let torrent_data = response.bytes().await.unwrap();
                    let save_root = CONFIG.get("save-root").unwrap().as_str().unwrap().to_string();
                    let save_path = format!("{save_root}/{}", id.clone());

                    match qb.lock().await.add_torrent(&AddTorrentArg::builder()
                        .source(TorrentSource::TorrentFiles {
                            torrents: vec![TorrentFile {
                                filename: id.clone(),
                                data: torrent_data.to_vec()
                            }]
                        })
                        .savepath(save_path)
                        .root_folder("true".to_string())
                        .sequential_download("true".to_string())
                        .build()
                    ).await {
                        Ok(()) => Ok::<String, Infallible>(r#"{"status": "success", "message": ""}"#.to_string()),
                        Err(e) => Ok(format!(r#"{{"status": "error", "message": "{e}"}}"#)),
                    }
                }
            }
        });

    let stop_download = warp::get()
        .and(warp::path!("stop"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let qb = Arc::clone(&qb);
            move |query: HashMap<String, String>| {
                let qb = Arc::clone(&qb);
                async move {
                    let token = query.get("token").unwrap().to_owned();
                    if token != correct_token {
                        return Ok("".to_string());
                    }
                    let hash = query.get("hash").unwrap().to_owned();
                    match qb.lock().await.delete_torrents(vec![hash], true).await {
                        Ok(()) => Ok::<String, Infallible>(r#"{"status": "success", "message": ""}"#.to_string()),
                        Err(e) => Ok(format!(r#"{{"status": "error", "message": "{e}"}}"#)),
                    }
                }
            }
        });

    let list_files = warp::get()
        .and(warp::path!("list"))
        .and(warp::query::<HashMap<String, String>>())
        .map(move |query: HashMap<String, String>| {
            let token = query.get("token").unwrap().to_owned();
            if token != correct_token {
                return "".to_string();
            }

            let save_root = Path::new(CONFIG.get("save-root").unwrap().as_str().unwrap());
            let rel_path = Path::new(query.get("path").unwrap());
            let mut full_path = save_root.join(rel_path);
            if !std::fs::metadata(&full_path).unwrap().is_dir() {
                full_path = full_path.parent().unwrap().to_path_buf();
            }
            let paths = std::fs::read_dir(full_path).unwrap();
            let local_files = LocalFileList {
                files: paths.map(|file| {
                    let full_path = file.as_ref().unwrap().path();
                    let rel_path = full_path.strip_prefix(save_root).unwrap();
                    LocalFile {
                        name: file.as_ref().unwrap().file_name().to_string_lossy().to_string(),
                        full_path: rel_path.to_string_lossy().to_string(),
                    }
                }).collect()
            };
            serde_json::to_string(&local_files).unwrap()
        });

    let sync = warp::get()
        .and(warp::path("sync"))
        .and(warp::fs::dir(CONFIG.get("save-root").unwrap().as_str().unwrap().to_string()));

    warp::serve(ping.or(search).or(torrent_list).or(download).or(stop_download).or(list_files).or(sync).with(warp::cors().allow_any_origin()))
        .run(([127, 0, 0, 1], 3000)).await;
}
