#![allow(dead_code)]


pub const KEY_MOD_LCTRL:u8=  0x01;
pub const KEY_MOD_LSHIFT:u8= 0x02;
pub const KEY_MOD_LALT:u8=   0x04;
pub const KEY_MOD_LMETA:u8=  0x08;
pub const KEY_MOD_RCTRL:u8=  0x10;
pub const KEY_MOD_RSHIFT:u8= 0x20;
pub const KEY_MOD_RALT:u8=   0x40;
pub const KEY_MOD_RMETA:u8=  0x80;

/**
 * Scan codes - last N slots in the HID report (usually 6).
 * 0x00; if no key pressed.
 * 
 * If more than N keys are pressed, the HID reports 
 * KEY_ERR_OVF in all slots to indicate this condition.
 */

pub const KEY_NONE:u8= 0x00; // No key:u8 pressed;
pub const KEY_ERR_OVF:u8= 0x01; //  Keyboard Error Roll Over - used for all slots if too many keys are pressed ("Phantom key:u8");
// 0x02; //  Keyboard POST Fail
// 0x03; //  Keyboard Error Undefined
pub const KEY_A:u8= 0x04; // Keyboard a and:u8 A;
pub const KEY_B:u8= 0x05; // Keyboard b and:u8 B;
pub const KEY_C:u8= 0x06; // Keyboard c and:u8 C;
pub const KEY_D:u8= 0x07; // Keyboard d and:u8 D;
pub const KEY_E:u8= 0x08; // Keyboard e and:u8 E;
pub const KEY_F:u8= 0x09; // Keyboard f and:u8 F;
pub const KEY_G:u8= 0x0a; // Keyboard g and:u8 G;
pub const KEY_H:u8= 0x0b; // Keyboard h and:u8 H;
pub const KEY_I:u8= 0x0c; // Keyboard i and:u8 I;
pub const KEY_J:u8= 0x0d; // Keyboard j and:u8 J;
pub const KEY_K:u8= 0x0e; // Keyboard k and:u8 K;
pub const KEY_L:u8= 0x0f; // Keyboard l and:u8 L;
pub const KEY_M:u8= 0x10; // Keyboard m and:u8 M;
pub const KEY_N:u8= 0x11; // Keyboard n and:u8 N;
pub const KEY_O:u8= 0x12; // Keyboard o and:u8 O;
pub const KEY_P:u8= 0x13; // Keyboard p and:u8 P;
pub const KEY_Q:u8= 0x14; // Keyboard q and:u8 Q;
pub const KEY_R:u8= 0x15; // Keyboard r and:u8 R;
pub const KEY_S:u8= 0x16; // Keyboard s and:u8 S;
pub const KEY_T:u8= 0x17; // Keyboard t and:u8 T;
pub const KEY_U:u8= 0x18; // Keyboard u and:u8 U;
pub const KEY_V:u8= 0x19; // Keyboard v and:u8 V;
pub const KEY_W:u8= 0x1a; // Keyboard w and:u8 W;
pub const KEY_X:u8= 0x1b; // Keyboard x and:u8 X;
pub const KEY_Y:u8= 0x1c; // Keyboard y and:u8 Y;
pub const KEY_Z:u8= 0x1d; // Keyboard z and:u8 Z;

pub const KEY_1:u8= 0x1e; // Keyboard 1 and:u8 !;
pub const KEY_2:u8= 0x1f; // Keyboard 2 and:u8 @;
pub const KEY_3:u8= 0x20; // Keyboard 3 and:u8 #;
pub const KEY_4:u8= 0x21; // Keyboard 4 and:u8 $;
pub const KEY_5:u8= 0x22; // Keyboard 5 and:u8 %;
pub const KEY_6:u8= 0x23; // Keyboard 6 and:u8 ^;
pub const KEY_7:u8= 0x24; // Keyboard 7 and:u8 &;
pub const KEY_8:u8= 0x25; // Keyboard 8 and:u8 *;
pub const KEY_9:u8= 0x26; // Keyboard 9 and:u8 (;
pub const KEY_0:u8= 0x27; // Keyboard 0 and:u8 );

pub const KEY_ENTER:u8= 0x28; // Keyboard Return (ENTER:u8);
pub const KEY_ESC:u8= 0x29; // Keyboard:u8 ESCAPE;
pub const KEY_BACKSPACE:u8= 0x2a; // Keyboard DELETE (Backspace:u8);
pub const KEY_TAB:u8= 0x2b; // Keyboard:u8 Tab;
pub const KEY_SPACE:u8= 0x2c; // Keyboard:u8 Spacebar;
pub const KEY_MINUS:u8= 0x2d; // Keyboard - and:u8 _;
pub const KEY_EQUAL:u8= 0x2e; // Keyboard = and:u8 +;
pub const KEY_LEFTBRACE:u8= 0x2f; // Keyboard [ and:u8 {;
pub const KEY_RIGHTBRACE:u8= 0x30; // Keyboard ] and:u8 };
pub const KEY_BACKSLASH:u8= 0x31; // Keyboard \ and:u8 |;
pub const KEY_HASHTILDE:u8= 0x32; // Keyboard Non-US # and:u8 ~;
pub const KEY_SEMICOLON:u8= 0x33; // Keyboard ; and:u8 :;
pub const KEY_APOSTROPHE:u8= 0x34; // Keyboard ' and:u8 ";
pub const KEY_GRAVE:u8= 0x35; // Keyboard ` and:u8 ~;
pub const KEY_COMMA:u8= 0x36; // Keyboard , and:u8 <;
pub const KEY_DOT:u8= 0x37; // Keyboard . and:u8 >;
pub const KEY_SLASH:u8= 0x38; // Keyboard / and:u8 ?;
pub const KEY_CAPSLOCK:u8= 0x39; // Keyboard Caps:u8 Lock;

pub const KEY_F1:u8= 0x3a; // Keyboard:u8 F1;
pub const KEY_F2:u8= 0x3b; // Keyboard:u8 F2;
pub const KEY_F3:u8= 0x3c; // Keyboard:u8 F3;
pub const KEY_F4:u8= 0x3d; // Keyboard:u8 F4;
pub const KEY_F5:u8= 0x3e; // Keyboard:u8 F5;
pub const KEY_F6:u8= 0x3f; // Keyboard:u8 F6;
pub const KEY_F7:u8= 0x40; // Keyboard:u8 F7;
pub const KEY_F8:u8= 0x41; // Keyboard:u8 F8;
pub const KEY_F9:u8= 0x42; // Keyboard:u8 F9;
pub const KEY_F10:u8= 0x43; // Keyboard:u8 F10;
pub const KEY_F11:u8= 0x44; // Keyboard:u8 F11;
pub const KEY_F12:u8= 0x45; // Keyboard:u8 F12;

pub const KEY_SYSRQ:u8= 0x46; // Keyboard Print:u8 Screen;
pub const KEY_SCROLLLOCK:u8= 0x47; // Keyboard Scroll:u8 Lock;
pub const KEY_PAUSE:u8= 0x48; // Keyboard:u8 Pause;
pub const KEY_INSERT:u8= 0x49; // Keyboard:u8 Insert;
pub const KEY_HOME:u8= 0x4a; // Keyboard:u8 Home;
pub const KEY_PAGEUP:u8= 0x4b; // Keyboard Page:u8 Up;
pub const KEY_DELETE:u8= 0x4c; // Keyboard Delete:u8 Forward;
pub const KEY_END:u8= 0x4d; // Keyboard:u8 End;
pub const KEY_PAGEDOWN:u8= 0x4e; // Keyboard Page:u8 Down;
pub const KEY_RIGHT:u8= 0x4f; // Keyboard Right:u8 Arrow;
pub const KEY_LEFT:u8= 0x50; // Keyboard Left:u8 Arrow;
pub const KEY_DOWN:u8= 0x51; // Keyboard Down:u8 Arrow;
pub const KEY_UP:u8= 0x52; // Keyboard Up:u8 Arrow;

pub const KEY_NUMLOCK:u8= 0x53; // Keyboard Num Lock and:u8 Clear;
pub const KEY_KPSLASH:u8= 0x54; // Keypad:u8 /;
pub const KEY_KPASTERISK:u8= 0x55; // Keypad:u8 *;
pub const KEY_KPMINUS:u8= 0x56; // Keypad:u8 -;
pub const KEY_KPPLUS:u8= 0x57; // Keypad:u8 +;
pub const KEY_KPENTER:u8= 0x58; // Keypad:u8 ENTER;
pub const KEY_KP1:u8= 0x59; // Keypad 1 and:u8 End;
pub const KEY_KP2:u8= 0x5a; // Keypad 2 and Down:u8 Arrow;
pub const KEY_KP3:u8= 0x5b; // Keypad 3 and:u8 PageDn;
pub const KEY_KP4:u8= 0x5c; // Keypad 4 and Left:u8 Arrow;
pub const KEY_KP5:u8= 0x5d; // Keypad:u8 5;
pub const KEY_KP6:u8= 0x5e; // Keypad 6 and Right:u8 Arrow;
pub const KEY_KP7:u8= 0x5f; // Keypad 7 and:u8 Home;
pub const KEY_KP8:u8= 0x60; // Keypad 8 and Up:u8 Arrow;
pub const KEY_KP9:u8= 0x61; // Keypad 9 and Page:u8 Up;
pub const KEY_KP0:u8= 0x62; // Keypad 0 and:u8 Insert;
pub const KEY_KPDOT:u8= 0x63; // Keypad . and:u8 Delete;

pub const KEY_102ND:u8= 0x64; // Keyboard Non-US \ and:u8 |;
pub const KEY_COMPOSE:u8= 0x65; // Keyboard:u8 Application;
pub const KEY_POWER:u8= 0x66; // Keyboard:u8 Power;
pub const KEY_KPEQUAL:u8= 0x67; // Keypad:u8 =;

pub const KEY_F13:u8= 0x68; // Keyboard:u8 F13;
pub const KEY_F14:u8= 0x69; // Keyboard:u8 F14;
pub const KEY_F15:u8= 0x6a; // Keyboard:u8 F15;
pub const KEY_F16:u8= 0x6b; // Keyboard:u8 F16;
pub const KEY_F17:u8= 0x6c; // Keyboard:u8 F17;
pub const KEY_F18:u8= 0x6d; // Keyboard:u8 F18;
pub const KEY_F19:u8= 0x6e; // Keyboard:u8 F19;
pub const KEY_F20:u8= 0x6f; // Keyboard:u8 F20;
pub const KEY_F21:u8= 0x70; // Keyboard:u8 F21;
pub const KEY_F22:u8= 0x71; // Keyboard:u8 F22;
pub const KEY_F23:u8= 0x72; // Keyboard:u8 F23;
pub const KEY_F24:u8= 0x73; // Keyboard:u8 F24;

pub const KEY_OPEN:u8= 0x74; // Keyboard:u8 Execute;
pub const KEY_HELP:u8= 0x75; // Keyboard:u8 Help;
pub const KEY_PROPS:u8= 0x76; // Keyboard:u8 Menu;
pub const KEY_FRONT:u8= 0x77; // Keyboard:u8 Select;
pub const KEY_STOP:u8= 0x78; // Keyboard:u8 Stop;
pub const KEY_AGAIN:u8= 0x79; // Keyboard:u8 Again;
pub const KEY_UNDO:u8= 0x7a; // Keyboard:u8 Undo;
pub const KEY_CUT:u8= 0x7b; // Keyboard:u8 Cut;
pub const KEY_COPY:u8= 0x7c; // Keyboard:u8 Copy;
pub const KEY_PASTE:u8= 0x7d; // Keyboard:u8 Paste;
pub const KEY_FIND:u8= 0x7e; // Keyboard:u8 Find;
pub const KEY_MUTE:u8= 0x7f; // Keyboard:u8 Mute;
pub const KEY_VOLUMEUP:u8= 0x80; // Keyboard Volume:u8 Up;
pub const KEY_VOLUMEDOWN:u8= 0x81; // Keyboard Volume:u8 Down;
// 0x82;  Keyboard Locking Caps Lock
// 0x83;  Keyboard Locking Num Lock
// 0x84;  Keyboard Locking Scroll Lock
pub const KEY_KPCOMMA:u8= 0x85; // Keypad:u8 Comma;
// 0x86;  Keypad Equal Sign
pub const KEY_RO:u8= 0x87; // Keyboard:u8 International1;
pub const KEY_KATAKANAHIRAGANA:u8= 0x88; // Keyboard:u8 International2;
pub const KEY_YEN:u8= 0x89; // Keyboard:u8 International3;
pub const KEY_HENKAN:u8= 0x8a; // Keyboard:u8 International4;
pub const KEY_MUHENKAN:u8= 0x8b; // Keyboard:u8 International5;
pub const KEY_KPJPCOMMA:u8= 0x8c; // Keyboard:u8 International6;
// 0x8d;  Keyboard International7
// 0x8e;  Keyboard International8
// 0x8f;  Keyboard International9
pub const KEY_HANGEUL:u8= 0x90; // Keyboard:u8 LANG1;
pub const KEY_HANJA:u8= 0x91; // Keyboard:u8 LANG2;
pub const KEY_KATAKANA:u8= 0x92; // Keyboard:u8 LANG3;
pub const KEY_HIRAGANA:u8= 0x93; // Keyboard:u8 LANG4;
pub const KEY_ZENKAKUHANKAKU:u8= 0x94; // Keyboard:u8 LANG5;
// 0x95;  Keyboard LANG6
// 0x96;  Keyboard LANG7
// 0x97;  Keyboard LANG8
// 0x98;  Keyboard LANG9
// 0x99;  Keyboard Alternate Erase
// 0x9a;  Keyboard SysReq/Attention
// 0x9b;  Keyboard Cancel
// 0x9c;  Keyboard Clear
// 0x9d;  Keyboard Prior
// 0x9e;  Keyboard Return
// 0x9f;  Keyboard Separator
// 0xa0;  Keyboard Out
// 0xa1;  Keyboard Oper
// 0xa2;  Keyboard Clear/Again
// 0xa3;  Keyboard CrSel/Props
// 0xa4;  Keyboard ExSel

// 0xb0;  Keypad 00
// 0xb1;  Keypad 000
// 0xb2;  Thousands Separator
// 0xb3;  Decimal Separator
// 0xb4;  Currency Unit
// 0xb5;  Currency Sub-unit
pub const KEY_KPLEFTPAREN:u8= 0xb6; // Keypad:u8 (;
pub const KEY_KPRIGHTPAREN:u8= 0xb7; // Keypad:u8 );
// 0xb8;  Keypad {
// 0xb9;  Keypad }
// 0xba;  Keypad Tab
// 0xbb;  Keypad Backspace
// 0xbc;  Keypad A
// 0xbd;  Keypad B
// 0xbe;  Keypad C
// 0xbf;  Keypad D
// 0xc0;  Keypad E
// 0xc1;  Keypad F
// 0xc2;  Keypad XOR
// 0xc3;  Keypad ^
// 0xc4;  Keypad %
// 0xc5;  Keypad <
// 0xc6;  Keypad >
// 0xc7;  Keypad &
// 0xc8;  Keypad &&
// 0xc9;  Keypad |
// 0xca;  Keypad ||
// 0xcb;  Keypad :
// 0xcc;  Keypad #
// 0xcd;  Keypad Space
// 0xce;  Keypad @
// 0xcf;  Keypad !
// 0xd0;  Keypad Memory Store
// 0xd1;  Keypad Memory Recall
// 0xd2;  Keypad Memory Clear
// 0xd3;  Keypad Memory Add
// 0xd4;  Keypad Memory Subtract
// 0xd5;  Keypad Memory Multiply
// 0xd6;  Keypad Memory Divide
// 0xd7;  Keypad +/-
// 0xd8;  Keypad Clear
// 0xd9;  Keypad Clear Entry
// 0xda;  Keypad Binary
// 0xdb;  Keypad Octal
// 0xdc;  Keypad Decimal
// 0xdd;  Keypad Hexadecimal

pub const KEY_LEFTCTRL:u8= 0xe0; // Keyboard Left:u8 Control;
pub const KEY_LEFTSHIFT:u8= 0xe1; // Keyboard Left:u8 Shift;
pub const KEY_LEFTALT:u8= 0xe2; // Keyboard Left:u8 Alt;
pub const KEY_LEFTMETA:u8= 0xe3; // Keyboard Left:u8 GUI;
pub const KEY_RIGHTCTRL:u8= 0xe4; // Keyboard Right:u8 Control;
pub const KEY_RIGHTSHIFT:u8= 0xe5; // Keyboard Right:u8 Shift;
pub const KEY_RIGHTALT:u8= 0xe6; // Keyboard Right:u8 Alt;
pub const KEY_RIGHTMETA:u8= 0xe7; // Keyboard Right:u8 GUI;

pub const KEY_MEDIA_PLAYPAUSE:u8= 0xe8;
pub const KEY_MEDIA_STOPCD:u8= 0xe9;
pub const KEY_MEDIA_PREVIOUSSONG:u8= 0xea;
pub const KEY_MEDIA_NEXTSONG:u8= 0xeb;
pub const KEY_MEDIA_EJECTCD:u8= 0xec;
pub const KEY_MEDIA_VOLUMEUP:u8= 0xed;
pub const KEY_MEDIA_VOLUMEDOWN:u8= 0xee;
pub const KEY_MEDIA_MUTE:u8= 0xef;
pub const KEY_MEDIA_WWW:u8= 0xf0;
pub const KEY_MEDIA_BACK:u8= 0xf1;
pub const KEY_MEDIA_FORWARD:u8= 0xf2;
pub const KEY_MEDIA_STOP:u8= 0xf3;
pub const KEY_MEDIA_FIND:u8= 0xf4;
pub const KEY_MEDIA_SCROLLUP:u8= 0xf5;
pub const KEY_MEDIA_SCROLLDOWN:u8= 0xf6;
pub const KEY_MEDIA_EDIT:u8= 0xf7;
pub const KEY_MEDIA_SLEEP:u8= 0xf8;
pub const KEY_MEDIA_COFFEE:u8= 0xf9;
pub const KEY_MEDIA_REFRESH:u8= 0xfa;
pub const KEY_MEDIA_CALC:u8= 0xfb;

