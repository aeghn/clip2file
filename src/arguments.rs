use clap::Parser;


#[derive(Parser, Default, Debug, Clone)]
#[command(author = "aeghn", version = "0.1", about = "save clipboard image into file")]
pub struct Arguments {
    #[clap(long, short, help = "Dir to save")]
    pub dir: String,
    #[clap(long, short, help = "File name to save")]
    pub name: Option<String>,
    #[clap(long, help = "Name file with timestamp")]
    pub timestamp: Option<bool>,
    #[clap(long, help = "base dir")]
    pub base_dir: Option<String>,
    #[clap(long, help = "parse text into file")]
    pub parse_text: Option<bool>,
    #[clap(long, help = "only image save, default false")]
    pub only_img: Option<bool>,
}
