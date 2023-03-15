use std::{
    fs,
    process::{exit, Command, Stdio},
    thread,
    time::Duration,
    io::{self, Read, Write},
    path::PathBuf
};

use image::{
    io::Reader, 
    GenericImageView, 
    Pixel
};

use rodio::{
    OutputStream,
    Sink,
    Decoder
};

const SHARPNESS: &[char] = &[' ','.','-','+','*','w','G','H','M','#','&','%'];

fn main() -> io::Result<()> {
    

    let entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    
    if entries.contains(&PathBuf::from(".\\cache")) {
        fs::remove_dir_all("./cache")?;
        fs::create_dir_all("./cache/frames")?;
    } else {
        fs::create_dir_all("./cache/frames")?;
    }

    if !entries.contains(&PathBuf::from(".\\config.txt")) {
        Config::create_file_config()?
    }
    
    let config = Config::new();

    extract_frames(&config);
    extract_audio(&config);

    let frame_files = fs::read_dir("./cache/frames")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    
    
    let mut frames = vec![String::with_capacity(config.width * config.height + config.height); frame_files.len() ];

    for (i, file) in frame_files.into_iter().enumerate() {
        let image = Reader::open(file.to_str().unwrap())
            .unwrap()
            .decode()
            .unwrap();
        
        
        for y in 0..config.height {
            for x in 0..config.width {
                let ch = *image.get_pixel(x as u32, y as u32).to_luma().0.get(0).unwrap() as f32 / 255.0;
                let ch = ch * (SHARPNESS.len() - 1) as f32;
                frames[i].push(SHARPNESS[ch as usize])
            } 
            frames[i].push('\n');
        }
        
    }

    thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
    
        let file = fs::File::open("cache/audio.wav").unwrap();
        sink.append(Decoder::new(io::BufReader::new(file)).unwrap());
        sink.set_volume(config.volume);
        sink.sleep_until_end();
    });

    
    for frame in frames.into_iter() {
        clear_scree();
        println!("{}", frame);
        thread::sleep(Duration::from_micros((1000000.0 / config.fps) as u64));
    }
    
    Ok(())
}

fn extract_frames(config: &Config) {
    Command::new("ffmpeg")
    .args(vec![
        "-i",
        &format!("{}", config.file),
        "-vf",
        "image2",
        "-vf",
        &format!("scale={}:{}", config.width, config.height),
        "./cache/frames/frame-%07d.png"
    ])
    .stdout(Stdio::null())
    .output()
    .unwrap_or_else(|e| {
        eprintln!("ffmpeg broken {}", e);
        exit(1);    
    });
}

fn extract_audio(config: &Config) {
    Command::new("ffmpeg")
    .args(vec![
        "-i",
        &format!("{}", config.file),
        "cache\\audio.wav"
    ])
    .stdout(Stdio::null())
    .output()
    .unwrap_or_else(|e| {
        eprintln!("ffmpeg broken {}", e);
        exit(1);    
    });
}

fn clear_scree() {
    Command::new("cmd")
        .args(vec!["/c", "cls"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
#[derive(Clone,Debug)]
struct Config {
    file: String,
    width: usize,
    height: usize,
    fps: f64,
    volume: f32
}

impl Config {
    fn new() -> Config {
        let mut file = fs::File::open("config.txt").expect("not found config file");
        let mut configs = String::new();
        file.read_to_string(&mut configs).expect("not reading");
        let configs: Vec<_> = configs.split_whitespace().collect();

        let mut file= String::new();
        let mut width = 0;
        let mut height = 0;
        let mut fps = 0.0;
        let mut volume = 0.0;

        for (i, config) in configs.iter().enumerate() {
            match *config {
                "video:" => file = configs[i + 1].to_string(),
                "width:" => width = configs[i + 1].parse().unwrap(),
                "height:" => height = configs[i + 1].parse().unwrap(),
                "fps:" => fps = configs[i + 1].parse().unwrap(),
                "volume:" => volume = configs[i + 1].parse().unwrap(),
                _ => continue
            }
        }

        Config {
            file,
            width,
            height,
            fps,
            volume
        }
         
    }
    
    fn create_file_config() -> io::Result<()> {
        let mut file = fs::File::create("config.txt")?;
        file.write_all(b"video: bad_apple.mp4\nwidth: 100\nheight: 20\nfps: 30\nvolume:0.1")?;
        Ok(())
    }
}
