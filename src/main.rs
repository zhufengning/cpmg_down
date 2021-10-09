use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use scraper::{Html, Selector};
use std::fs;
use std::path::PathBuf;

const KEY: &str = "xxxmanga.woo.key";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    match std::env::args().nth(1) {
        Some(mg_name) => {
            match std::env::args().nth(2) {
                Some(range) => {
                    let range: Vec<&str> = range.split(",").collect();
                    let url_chapters =
                        format!("https://www.copymanga.com/comicdetail/{}/chapters", mg_name);
                    let res_json: serde_json::Value = reqwest::Client::new()
                        .get(url_chapters)
                        .send()
                        .await?
                        .json()
                        .await?;
                    let code = res_json["code"].as_i64().unwrap_or_default();
                    if code != 200 {
                        println!("{}", res_json["message"].as_str().unwrap_or_default());
                        return Ok(());
                    }
                    let shit = res_json["results"].as_str().unwrap_or_default();
                    let shit = &unshit(shit);
                    let chapers: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&shit);
                    match chapers {
                        Ok(chapters) => {
                            let chapters = chapters["groups"]["default"]["chapters"]
                                .as_array()
                                .unwrap_or(Box::leak(vec![].into()));

                            if let Err(e) = fs::create_dir(&mg_name) {
                                println!("{}", e);
                                println!("输入y继续运行");
                                let x: String;
                                text_io::scan!("{}", x);
                                if x != "y" {
                                    return Ok(());
                                }
                            }

                            for v in range.iter() {
                                let v: Vec<&str> = v.split("-").collect();
                                if v.len() == 2 {
                                    let mut i: i32 = v[0].parse().unwrap_or_default();
                                    let ed: i32 = v[1].parse().unwrap_or_default();
                                    while i <= ed {
                                        down(i, chapters.clone(), &mg_name).await;
                                        i += 1;
                                    }
                                } else {
                                    down(
                                        v[0].parse().unwrap_or_default(),
                                        chapters.clone(),
                                        &mg_name,
                                    )
                                    .await;
                                }
                            }
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
                None => {
                    println!("参数少了");
                }
            };
            Ok(())
        }
        None => {
            println!("莫得参数");
            Ok(())
        }
    }
}

fn unshit(shit: &str) -> String {
    let iv = shit[..16].as_bytes();
    let shit_back: String = String::from(&shit[16..]);
    let mut new_shit: Vec<u8> = hex::decode(shit_back).unwrap_or_default();
    let key = KEY.as_bytes();
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_from_slices(key, iv);
    match cipher {
        Ok(cipher) => String::from(
            std::str::from_utf8(cipher.decrypt(&mut new_shit).unwrap_or_default())
                .unwrap_or_default(),
        ),
        Err(e) => {
            println!("{:?}", e);
            format!("")
        }
    }
}

async fn down(p: i32, chapters: Vec<serde_json::Value>, name: &str) {
    println!("第{}节开始下载", p + 1);
    let chapters_len_len = get_len(chapters.len() as i32 + 1);
    let mut ps = mkzero(chapters_len_len - get_len(p as i32));
    ps.push_str(&p.to_string());
    let mut p_f = PathBuf::new();
    p_f.push(name);
    p_f.push(ps);
    if let Err(e) = fs::create_dir(p_f) {
        println!("{}  但可能没事儿", e);
    }
    let p: usize = p as usize - 1;
    let chapter_id = chapters[p]["id"].as_str().unwrap_or_default();
    let url_view = format!(
        "https://www.copymanga.com/comic/{}/chapter/{}",
        name, chapter_id
    );
    println!("获取图片列表");
    let pics = reqwest::Client::new().get(url_view).send().await;
    if let Ok(pics) = pics {
        let pics = pics.text().await;
        if let Ok(pics) = pics {
            let doc = Html::parse_document(&pics);
            if let Ok(sec) = Selector::parse("div[class=\"imageData\"") {
                if let Some(img_data) = doc.select(&sec).next() {
                    let coded_json = img_data.value().attr("contentkey").unwrap_or_default();
                    let s_json: serde_json::Value =
                        serde_json::from_str(&*unshit(coded_json)).unwrap_or_default();
                    let pics: &Vec<serde_json::Value> =
                        s_json.as_array().unwrap_or(Box::leak(vec![].into()));
                    let pics_len_len = get_len(pics.len() as i32 + 1);
                    for (i, v) in pics.iter().enumerate() {
                        let mut file_name = PathBuf::new();
                        if let Some(u) = std::env::args().nth(3) {
                            file_name = PathBuf::from(u);
                        }
                        file_name.push(name);
                        let mut next_path = String::new();
                        next_path.push_str(&mkzero(chapters_len_len - get_len((p + 1) as i32)));
                        let tmp = (p + 1).to_string();
                        next_path.push_str(&tmp);
                        file_name.push(next_path);
                        let mut next_path = String::new();
                        next_path.push_str(&mkzero(pics_len_len - get_len((i + 1) as i32)));
                        let tmp = (i + 1).to_string();
                        next_path.push_str(&tmp);
                        next_path.push_str(".jpg");
                        file_name.push(next_path);
                        print!("{}下载中。。。  ", file_name.as_path().to_str().unwrap());
                        match reqwest::Client::new()
                            .get(v["url"].as_str().unwrap_or_default())
                            .send()
                            .await
                        {
                            Ok(res) => match res.bytes().await {
                                Ok(res) => match fs::write(file_name, res) {
                                    Ok(()) => {
                                        println!("下载成功");
                                    }
                                    Err(e) => {
                                        println!("下载失败:{}", e);
                                    }
                                },
                                Err(e) => {
                                    println!("下载失败:{}", e);
                                }
                            },
                            Err(e) => {
                                println!("下载失败:{}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_len(num: i32) -> i32 {
    let mut num_len = 0;
    let mut len = num;
    while len > 0 {
        num_len += 1;
        len /= 10;
    }
    num_len
}

fn mkzero(num: i32) -> String {
    let mut num = num;
    let mut s = String::new();
    while num > 0 {
        s.push('0');
        num -= 1;
    }
    s
}
