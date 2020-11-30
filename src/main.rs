#[macro_use]
extern crate clap;

use bracket_noise::prelude::*;
use gif::{Encoder, Frame, Repeat};
use rand::random;
use std::{
    borrow::Cow,
    collections::VecDeque,
    fs::File,
    sync::{Arc, Mutex},
    thread,
    time::SystemTime,
};

static SCALE: f32 = 0.014;

fn main() {
    let mut noise: FastNoise = FastNoise::new();

    let matches = clap_app!(NoiseGen =>
        (version: "0.1 BETA"                                                                      )
        (author: "LordClips <lordclips@protonmail.com>"                                           )
        (about: "Generates noise gifs, animated or otherwise."                                    )
        (@arg SEED:              -s --seed         +takes_value "Sets custom seed."               )
        (@arg INTERP:            -I --interp       +takes_value "Sets interpolation."             )
        (@arg NFREQUENCY:        -F --frequency    +takes_value "Sets frequency."                 )
        (@arg NTYPE:             -n --ntype        +takes_value "Sets noise type."                )
        (@arg OCTAVES:           -o --octaves      +takes_value "Sets fractal octaves."           )
        (@arg LACUNARITY:        -l --lacunarity   +takes_value "Sets fractal lacunarity."        )
        (@arg GAIN:              -g --gain         +takes_value "Sets fractal gain."              )
        (@arg TYPE:              -f --ftype        +takes_value "Sets fractal type."              )
        (@arg DIST:              -d --dist         +takes_value "Sets cellular distance."         )
        (@arg DIST_FUNC:         -D --func         +takes_value "Sets cellular distance function.")
        (@arg RET_TYPE:          -r --ret-type     +takes_value "Sets cellular return type."      )
        (@arg JITTER:            -j --jitter       +takes_value "Sets cell jitter."               )
        (@arg DIST_INDICES:      -i --indices      +takes_value "Sets cell distance indices."     )
        (@arg GRAD_PERTERB_AMP:  -p --perterb-amp  +takes_value "Sets gradient perterb amp."      )
        (@arg WIDTH:             +required "Sets width."                                          )
        (@arg HEIGHT:            +required "Sets height."                                         )
        (@arg FRAMES:            +required "Sets frame count."                                    )
    ).get_matches();

    if let Ok(seed) = value_t!(matches.value_of("SEED"), u64) {
        noise.set_seed(seed);
    } else {
        let seed: u64 = random();
        noise.set_seed(seed);
    }

    let width: u16 = value_t!(matches.value_of("WIDTH"), u16).unwrap();
    let height: u16 = value_t!(matches.value_of("HEIGHT"), u16).unwrap();
    let frame_count: u32 = value_t!(matches.value_of("FRAMES"), u32).unwrap();

    if let Some(interp) = matches.value_of("INTERP") {
        match interp {
            "linear" => noise.set_interp(Interp::Linear),
            "hermite" => noise.set_interp(Interp::Hermite),
            "quintic" => noise.set_interp(Interp::Quintic),
            _ => noise.set_interp(Interp::Hermite),
        }
    } else {
        noise.set_interp(Interp::Hermite);
    }

    /*
    noise.set_interp(Interp::Quintic);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_octaves(4);
    noise.set_fractal_lacunarity(2.0);
    noise.set_fractal_type(FractalType::FBM);
    */

    let mut color_map = Vec::new();

    for val in 0..=255 {
        color_map.extend_from_slice(&[val, val, val]);
    }

    let frame_deque: Arc<Mutex<VecDeque<Frame>>> = Arc::new(Mutex::new(VecDeque::new()));
    let clone_deque = frame_deque.clone();
    let begin = SystemTime::now();

    thread::spawn(move || {
        for z in 0..frame_count {
            let mut frame = Frame::default();

            frame.width = width;
            frame.height = height;

            let mut state = Vec::new();

            for y in 0..height {
                for x in 0..width {
                    let _x = x as f32 * SCALE;
                    let _y = y as f32 * SCALE;
                    let _z = z as f32 * SCALE;

                    let val = noise.get_noise3d(_x, _y, _z) * 0.5 + 0.5;

                    state.push((val * 255.) as u8);
                }
            }

            frame.buffer = Cow::Owned(state);

            let mut fd = clone_deque.lock().unwrap();

            println!("Frame {} created.", z);

            fd.push_back(frame);
        }
    });

    let mut inc = 0;

    let image: File = File::create("out.gif").unwrap();
    let mut encoder = Encoder::new(image, width, height, &color_map).unwrap();

    encoder.set_repeat(Repeat::Infinite).unwrap();

    let wrapped_encoder = Arc::new(Mutex::new(encoder));

    let encode_thread = thread::spawn(move || loop {
        if inc == frame_count {
            break;
        }

        let mut fd = frame_deque.lock().unwrap();

        match fd.pop_front() {
            None => {}
            Some(frame) => {
                let mut enc = wrapped_encoder.lock().unwrap();

                enc.write_frame(&frame).unwrap();

                println!("Encoding frame: {}", inc);

                inc += 1;
            }
        };
    });

    encode_thread.join().unwrap();

    println!(
        "Time: {}.{}",
        begin.elapsed().unwrap().as_secs(),
        begin.elapsed().unwrap().subsec_millis()
    );
}
