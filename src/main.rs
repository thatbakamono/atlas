use std::{collections::HashMap, fs, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use image::GenericImageView;
use serde::Serialize;

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Command::Generate {
                files,
                atlas_output,
                metadata_output,
                width,
                height,
                algorithm,
            } => {
                let images = files
                    .into_iter()
                    .map(|file| (file.clone(), image::open(file).unwrap()))
                    .collect::<Vec<_>>();

                let mut atlas = image::RgbaImage::new(width, height);
                let mut fragments = HashMap::new();

                match algorithm {
                    Algorithm::Etagere => {
                        let mut allocator = etagere::AtlasAllocator::new(etagere::size2(
                            width as i32,
                            height as i32,
                        ));

                        for (file_path, image) in images {
                            let allocation = allocator
                                .allocate(etagere::size2(
                                    image.width() as i32,
                                    image.height() as i32,
                                ))
                                .expect("Failed to allocate atlas space");

                            let rectangle = allocation.rectangle;

                            image.pixels().for_each(|(x, y, pixel)| {
                                atlas.put_pixel(
                                    rectangle.min.x as u32 + x,
                                    rectangle.min.y as u32 + y,
                                    pixel,
                                );
                            });

                            fragments.insert(
                                file_path.clone(),
                                Fragment {
                                    center: Vector2::new(
                                        (rectangle.center().x
                                            - (rectangle.width() - image.width() as i32) / 2)
                                            as f32,
                                        (rectangle.center().y
                                            - (rectangle.height() - image.height() as i32) / 2)
                                            as f32,
                                    ),
                                    size: Vector2::new(image.width() as f32, image.height() as f32),
                                },
                            );
                        }
                    }
                    Algorithm::Guillotiere => {
                        let mut allocator = guillotiere::AtlasAllocator::new(guillotiere::size2(
                            width as i32,
                            height as i32,
                        ));

                        for (file_path, image) in images {
                            let allocation = allocator
                                .allocate(guillotiere::size2(
                                    image.width() as i32,
                                    image.height() as i32,
                                ))
                                .expect("Failed to allocate atlas space");

                            let rectangle = allocation.rectangle;

                            image.pixels().for_each(|(x, y, pixel)| {
                                atlas.put_pixel(
                                    rectangle.min.x as u32 + x,
                                    rectangle.min.y as u32 + y,
                                    pixel,
                                );
                            });

                            fragments.insert(
                                file_path.clone(),
                                Fragment {
                                    center: Vector2::new(
                                        (rectangle.center().x
                                            - (rectangle.width() - image.width() as i32) / 2)
                                            as f32,
                                        (rectangle.center().y
                                            - (rectangle.height() - image.height() as i32) / 2)
                                            as f32,
                                    ),
                                    size: Vector2::new(image.width() as f32, image.height() as f32),
                                },
                            );
                        }
                    }
                }

                atlas.save(&atlas_output).unwrap();

                fs::write(
                    metadata_output,
                    serde_json::to_string_pretty(&fragments).unwrap(),
                )
                .unwrap();
            }
        }
    } else {
        println!("No command specified");
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Clone)]
enum Command {
    Generate {
        #[arg(short, long, num_args = 1..)]
        files: Vec<PathBuf>,
        #[arg(short, long)]
        atlas_output: PathBuf,
        #[arg(short, long)]
        metadata_output: PathBuf,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long, value_enum, default_value_t = Algorithm::Etagere)]
        algorithm: Algorithm,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Algorithm {
    Etagere,
    Guillotiere,
}

#[derive(Serialize)]
struct Fragment {
    center: Vector2,
    size: Vector2,
}

#[derive(Serialize)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
