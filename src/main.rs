const KEY: &str = "xxxmanga.woo.key";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    match std::env::args().nth(1) {
        Some(u) => {
            let url_chapters = format!("https://www.copymanga.com/comicdetail/{}/chapters", u);
            let res_json:serde_json::Value = reqwest::Client::new()
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
            unsafe{
                println!("{}", unshit(shit));
            }
            Ok(())
        }
        None => {
            println!("莫得参数");
            Ok(())
        }
    }
}

unsafe fn unshit(shit: &str) -> String {
    let shit_front = shit[..16].as_bytes();
    let mut iv = [0u8;16];
    let mut blank_cnt = 0;
    while blank_cnt < 16 {
        iv[blank_cnt] = shit_front[blank_cnt];
        blank_cnt += 1;
    }
    let mut cipher = crypto2::blockmode::Aes128Cbc::new(&KEY.as_bytes());
    let shit_back:String = String::from(&shit[16..]);
    let mut new_shit: Vec<u8> = hex::decode(shit_back).unwrap_or_default();
    while new_shit.len()% 128 != 0 {
        new_shit.push(0);
    }
    cipher.decrypt(&iv, &mut new_shit);
    println!("{:?}", new_shit);
    println!("{}", blank_cnt);
    println!("{} {}", new_shit.len(), new_shit.len() % 128);
    return String::from(std::str::from_utf8_unchecked(&new_shit));
}
