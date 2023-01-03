
use usbd_human_interface_device::page::Keyboard as K;

use crate::{keyboard::{keyboard::{KeyboardTrait, Ninja}, key::Key}, Keys};

impl KeyboardTrait for Ninja {
    const COLS:usize=6;
    const ROWS:usize=4;

    fn get_default_keys()->Keys{
        [
            [
                [
                    [Key::Code(K::Escape),Key::Code(K::Q),Key::Code(K::W),Key::Code(K::E),Key::Code(K::R),Key::Code(K::T)],
                    [Key::Code(K::Tab),Key::Code(K::A),Key::Code(K::S),Key::Code(K::D),Key::Code(K::F),Key::Code(K::G)],
                    [Key::Code(K::LeftShift),Key::Code(K::Z),Key::Code(K::X),Key::Code(K::C),Key::Code(K::V),Key::Code(K::B)],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::Code(K::LeftControl),Key::Code(K::LeftGUI),Key::Layer ],
                ],
                [
                    [Key::Code(K::F1),Key::Code(K::F2),Key::Code(K::F3),Key::Code(K::F4),Key::Code(K::F5),Key::Code(K::F6) ],
                    [Key::Code(K::Keyboard1),Key::Code(K::Keyboard1),Key::Code(K::Keyboard2),Key::Code(K::Keyboard3),Key::Code(K::Keyboard4),Key::Code(K::Keyboard5)],
                    [Key::Code(K::Backslash),Key::Code(K::Z),Key::Code(K::X) ,Key::Code(K::C), Key::Code(K::V) ,Key::Code(K::B)  ],
                    [Key::NoKey,Key::NoKey,Key::NoKey ,Key::Code(K::LeftAlt),Key::Code(K::RightGUI),Key::Layer ],
                ],
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
                ,
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
            ],
            [
                [
                    [Key::Code(K::Y), Key::Code(K::U),Key::Code(K::I),Key::Code(K::O),Key::Code(K::P),Key::Code(K::DeleteBackspace) ],
                    [Key::Code(K::H), Key::Code(K::J),Key::Code(K::K),Key::Code(K::L),Key::Code(K::Semicolon),Key::Code(K::Backslash) ],
                    [Key::Code(K::N), Key::Code(K::M),Key::Code(K::Comma),Key::Code(K::LeftBrace),Key::Code(K::RightBrace),Key::Code(K::Apostrophe) ],
                    [Key::Code(K::ReturnEnter),Key::Code(K::Space),Key::Code(K::Dot) ,Key::NoKey,Key::NoKey,Key::NoKey]
                ],
                [
                    [Key::Code(K::F7)    ,Key::Code(K::F8)  ,Key::Code(K::F9)    ,Key::Code(K::F10)   ,Key::Code(K::F11)     ,Key::Code(K::F12) ],
                    [Key::Code(K::Keyboard6),Key::Code(K::UpArrow)  ,Key::Code(K::Keyboard7),Key::Code(K::Keyboard8),Key::Code(K::Keyboard9),Key::Code(K::Keyboard0)],
                    [Key::Code(K::LeftArrow)  ,Key::Code(K::DownArrow),Key::Code(K::RightArrow) ,Key::Code(K::PageUp),Key::Code(K::PageDown),Key::Code(K::Minus) ],
                    [Key::Code(K::DeleteForward),Key::Code(K::Home),Key::Code(K::End)   ,Key::NoKey           ,Key::NoKey              ,Key::NoKey]
                ],
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
                ,
                [
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                    [Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey,Key::NoKey],
                ]
            ]
        ]
    }
}