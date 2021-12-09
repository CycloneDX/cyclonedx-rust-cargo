/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 * Copyright (c) OWASP Foundation. All Rights Reserved.
 */
/**
* A special acknowledgement Ossi Herrala from SensorFu for providing a
* starting point in which to develop this plugin. The original project
* this plugin was derived from is sensorfu/cargo-bom v0.3.1 (MIT licensed)
* https://github.com/sensorfu/cargo-bom
*
* MIT License
*
* Copyright (c) 2017-2019 SensorFu Oy
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/
use cargo_cyclonedx::generator::{Generator, SbomGenerator};
use cargo_cyclonedx::traits::ToXml;
use std::{
    fs::File,
    io::{self, LineWriter},
    path::PathBuf,
};

use anyhow::Result;
use env_logger::Builder;
use log::{LevelFilter, SetLoggerError};
use structopt::StructOpt;
use xml_writer::XmlWriter;

mod format;
use format::Format;

mod cli;
use cli::{Args, Opts};

fn main() -> anyhow::Result<()> {
    let Opts::Bom(args) = Opts::from_args();
    setup_logging(&args)?;
    real_main(args)
}

fn real_main(args: Args) -> anyhow::Result<()> {
    let manifest = locate_manifest(&args)?;
    let generator = SbomGenerator { all: args.all };

    log::trace!("SBOM generation started");
    let boms = generator.create_sboms(manifest)?;
    log::trace!("SBOM generation finished");

    match args.format {
        Format::Json => {
            for (manifest_path, bom) in boms {
                let path = manifest_path.with_file_name("bom.json");
                log::info!("Outputting {}", path.display());
                serde_json::to_writer_pretty(File::create(path)?, &bom)
                    .map_err(anyhow::Error::from)?;
            }
        }
        Format::Xml => {
            for (manifest_path, bom) in boms {
                let path = manifest_path.with_file_name("bom.xml");
                log::info!("Outputting {}", path.display());
                let file = File::create(path)?;
                let file = LineWriter::new(file);
                let mut xml = XmlWriter::new(file);

                bom.to_xml(&mut xml)?;
                xml.close()?;
                xml.flush()?;
                let _actual = xml.into_inner();
            }
        }
    }

    Ok(())
}

fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    let mut builder = Builder::new();

    // default cargo internals to quiet unless overriden via an environment variable
    // call with RUST_LOG='cargo::=debug' to access these logs
    builder.filter_module("cargo::", LevelFilter::Error);

    let level_filter = if args.quiet {
        LevelFilter::Off
    } else {
        match args.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };
    builder.filter_level(level_filter);

    builder.parse_default_env(); // allow overriding CLI arguments
    builder.try_init()?;
    Ok(())
}

fn locate_manifest(args: &Args) -> Result<PathBuf, io::Error> {
    if let Some(manifest_path) = &args.manifest_path {
        let manifest_path = manifest_path.canonicalize()?;
        log::info!(
            "Using manually specified Cargo.toml manifest located at: {}",
            manifest_path.to_string_lossy()
        );
        Ok(manifest_path.clone())
    } else {
        let manifest_path = std::env::current_dir()?.join("Cargo.toml");
        log::info!(
            "Using Cargo.toml manifest located at: {}",
            manifest_path.to_string_lossy()
        );
        Ok(manifest_path)
    }
}
