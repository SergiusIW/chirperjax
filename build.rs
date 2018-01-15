// gate_demo, a demo game built using the "gate" game library.
// Copyright (C) 2017-2018  Matthew D. Michelotti
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

extern crate gate_build;

use std::path::Path;
use std::env;

use gate_build::AssetPacker;

// build script packs image assets into atlases and generates enums to reference assets

fn main() {
    let is_wasm = env::var("TARGET").map(|t| t.starts_with("wasm32")).unwrap_or(false);
    let out_dir = env::var("OUT_DIR").unwrap();
    let gen_code_path = Path::new(&out_dir).join("asset_id.rs");

    let assets_dir = if is_wasm { "html" } else { "assets" };
    let mut packer = AssetPacker::new(Path::new(assets_dir));
    packer.cargo_rerun_if_changed();
    packer.sprites(Path::new("src_assets/sprites"));
    packer.tiles(Path::new("src_assets/tiles"));
    packer.music(Path::new("src_assets/music"));
    packer.sounds(Path::new("src_assets/sounds"));
    if is_wasm { packer.gen_javascript_and_html(); }
    packer.gen_asset_id_code(&gen_code_path);
}
