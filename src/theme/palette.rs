use bevy::prelude::*;
use std::sync::LazyLock;

pub static LABEL_TEXT: LazyLock<Color> = LazyLock::new(|| Srgba::hex("#ddd369").unwrap().into());

pub static HEADER_TEXT: LazyLock<Color> = LazyLock::new(|| Srgba::hex("#fcfbcc").unwrap().into());

pub static BUTTON_TEXT: LazyLock<Color> = LazyLock::new(|| Srgba::hex("#ececec").unwrap().into());

pub static BUTTON_BACKGROUND: LazyLock<Color> =
    LazyLock::new(|| Srgba::hex("#4666bf").unwrap().into());

pub static BUTTON_HOVERED_BACKGROUND: LazyLock<Color> =
    LazyLock::new(|| Srgba::hex("#6299d1").unwrap().into());

pub static BUTTON_PRESSED_BACKGROUND: LazyLock<Color> =
    LazyLock::new(|| Srgba::hex("#3d4999").unwrap().into());

pub static TRANSPARENT: LazyLock<Color> = LazyLock::new(|| Srgba::hex("#00000000").unwrap().into());
