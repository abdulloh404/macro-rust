//! บันทึกจังหวะที่ผู้ใช้กดปุ่มสำหรับเทียบกับ macro โดยเริ่มด้วย F9 และหยุดด้วย F10

use crate::xdotool_key_name_from_rdev_key;
use rdev::{listen, Button, EventType, Key};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{self, Write},
    time::Instant,
};

#[cfg(test)]
mod tests;

pub(super) fn run() {
    let mut trace = ManualTrace::default();
    println!("Manual timing: F9 = Start recording, F10 = Stop recording, Ctrl+C = Exit");
    if let Err(error) = listen(move |event| {
        let now = Instant::now();
        let _ = trace.handle_event(event.event_type, now, io::stderr());
    }) {
        eprintln!("Manual timing listener error: {error:?}");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum InputTarget {
    Key(&'static str),
    MouseButton(&'static str),
}

/// เก็บเวลาเริ่มบันทึกและเวลาที่กดปุ่มแต่ละปุ่มเพื่อคำนวณระยะกดค้างตอนปล่อย
#[derive(Default)]
struct ManualTrace {
    started_at: Option<Instant>,
    held_inputs: HashMap<InputTarget, Instant>,
}

impl ManualTrace {
    fn handle_event(
        &mut self,
        event: EventType,
        now: Instant,
        mut output: impl Write,
    ) -> io::Result<()> {
        if event == EventType::KeyPress(Key::F9) && self.started_at.is_none() {
            self.started_at = Some(now);
            return writeln!(output, "manual_trace state=started");
        }

        let Some(started_at) = self.started_at else {
            return Ok(());
        };

        if event == EventType::KeyPress(Key::F10) {
            self.started_at = None;
            self.held_inputs.clear();
            return writeln!(output, "manual_trace state=stopped");
        }

        let (input, pressed) = match event {
            EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                let Some(target) = manual_key_name(key) else {
                    return Ok(());
                };
                (
                    InputTarget::Key(target),
                    matches!(event, EventType::KeyPress(_)),
                )
            }
            EventType::ButtonPress(button) | EventType::ButtonRelease(button) => {
                let target = match button {
                    Button::Left => "1",
                    Button::Right => "3",
                    _ => return Ok(()),
                };
                (
                    InputTarget::MouseButton(target),
                    matches!(event, EventType::ButtonPress(_)),
                )
            }
            _ => return Ok(()),
        };

        let at_ms = now.duration_since(started_at).as_secs_f64() * 1_000.0;
        let (action, target) = match (input, pressed) {
            (InputTarget::Key(target), true) => ("keydown", target),
            (InputTarget::Key(target), false) => ("keyup", target),
            (InputTarget::MouseButton(target), true) => ("mousedown", target),
            (InputTarget::MouseButton(target), false) => ("mouseup", target),
        };
        if pressed {
            // เก็บเวลา `keydown` ครั้งแรกแม้ได้รับ `KeyPress` ซ้ำก่อน `KeyRelease`
            if let Entry::Vacant(entry) = self.held_inputs.entry(input) {
                entry.insert(now);
                writeln!(
                    output,
                    "input action={action:?} target={target:?} at_ms={at_ms:.3} source=manual"
                )?;
            }
        } else if let Some(pressed_at) = self.held_inputs.remove(&input) {
            let held_ms = now.duration_since(pressed_at).as_secs_f64() * 1_000.0;
            writeln!(output, "input action={action:?} target={target:?} at_ms={at_ms:.3} source=manual held_ms={held_ms:.3}")?;
        }
        Ok(())
    }
}

/// ใช้ชื่อปุ่มตรงกับ `xdotool` รวมถึงปุ่มตัวเลขที่ใช้กด buff
fn manual_key_name(key: Key) -> Option<&'static str> {
    match key {
        Key::Num1 => Some("1"),
        Key::Num2 => Some("2"),
        Key::Num3 => Some("3"),
        _ => xdotool_key_name_from_rdev_key(key),
    }
}
