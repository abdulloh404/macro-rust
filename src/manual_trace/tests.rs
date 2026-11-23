//! ตรวจช่วงบันทึกและเวลาค้างของ input ด้วย `Instant` ที่กำหนดใน test

use super::ManualTrace;
use rdev::{
    Button,
    EventType::{self, ButtonPress, ButtonRelease, KeyPress, KeyRelease},
    Key,
};
use std::time::{Duration, Instant};

fn record_events(events: &[(u64, EventType)]) -> String {
    let mut trace = ManualTrace::default();
    let start = Instant::now();
    let mut output = Vec::new();
    for &(offset_ms, event) in events {
        trace
            .handle_event(event, start + Duration::from_millis(offset_ms), &mut output)
            .unwrap();
    }
    String::from_utf8(output).unwrap()
}

#[test]
fn captures_only_between_start_and_stop_and_resets_for_the_next_capture() {
    let output = record_events(&[
        (90, KeyPress(Key::KeyS)),
        (100, KeyPress(Key::F9)),
        (101, KeyRelease(Key::KeyS)),
        (105, KeyPress(Key::F9)),
        (110, KeyPress(Key::KeyS)),
        (120, KeyPress(Key::F10)),
        (130, KeyRelease(Key::KeyS)),
        (200, KeyPress(Key::F9)),
        (205, KeyRelease(Key::KeyS)),
        (210, KeyPress(Key::KeyS)),
        (250, KeyRelease(Key::KeyS)),
        (260, KeyPress(Key::F10)),
    ]);

    assert_eq!(
        output,
        concat!(
            "manual_trace state=started\n",
            "input action=\"keydown\" target=\"s\" at_ms=10.000 source=manual\n",
            "manual_trace state=stopped\n",
            "manual_trace state=started\n",
            "input action=\"keydown\" target=\"s\" at_ms=10.000 source=manual\n",
            "input action=\"keyup\" target=\"s\" at_ms=50.000 source=manual held_ms=40.000\n",
            "manual_trace state=stopped\n",
        )
    );
}

#[test]
fn measures_key_and_mouse_holds_independently_for_the_same_target_label() {
    let output = record_events(&[
        (0, KeyPress(Key::F9)),
        (10, KeyPress(Key::Num1)),
        (15, ButtonPress(Button::Left)),
        (35, ButtonRelease(Button::Left)),
        (55, KeyRelease(Key::Num1)),
        (70, ButtonPress(Button::Right)),
        (85, ButtonRelease(Button::Right)),
    ]);

    assert_eq!(
        output,
        concat!(
            "manual_trace state=started\n",
            "input action=\"keydown\" target=\"1\" at_ms=10.000 source=manual\n",
            "input action=\"mousedown\" target=\"1\" at_ms=15.000 source=manual\n",
            "input action=\"mouseup\" target=\"1\" at_ms=35.000 source=manual held_ms=20.000\n",
            "input action=\"keyup\" target=\"1\" at_ms=55.000 source=manual held_ms=45.000\n",
            "input action=\"mousedown\" target=\"3\" at_ms=70.000 source=manual\n",
            "input action=\"mouseup\" target=\"3\" at_ms=85.000 source=manual held_ms=15.000\n",
        )
    );
}

#[test]
fn ignores_unsupported_keys_and_repeated_press_events() {
    let output = record_events(&[
        (0, KeyPress(Key::F9)),
        (1, KeyPress(Key::Backspace)),
        (3, KeyRelease(Key::Backspace)),
        (10, KeyPress(Key::KeyS)),
        (20, KeyPress(Key::KeyS)),
        (35, KeyRelease(Key::KeyS)),
        (40, KeyRelease(Key::KeyS)),
    ]);

    assert_eq!(
        output,
        concat!(
            "manual_trace state=started\n",
            "input action=\"keydown\" target=\"s\" at_ms=10.000 source=manual\n",
            "input action=\"keyup\" target=\"s\" at_ms=35.000 source=manual held_ms=25.000\n",
        )
    );
}
