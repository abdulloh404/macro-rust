//! ส่งคำสั่งคีย์บอร์ดและเมาส์ของ macro ผ่าน `xdotool`
//! ตั้ง `SKILLBOT_TRACE_INPUT=1` เพื่อบันทึกเวลาและผลของแต่ละคำสั่งทาง `stderr`

use std::{
    io::{self, Write},
    process::{Command, ExitStatus},
    time::{Duration, Instant},
};

/// วัดเวลาตั้งแต่เริ่มเรียก `xdotool` จน process จบเมื่อเปิด input trace
pub(super) fn run_xdotool_command(action: &str, target: &str, trace_started_at: Option<Instant>) {
    let timing = trace_started_at.map(|trace_start| (trace_start, Instant::now()));
    let result = Command::new("xdotool").args([action, target]).status();

    if let Some((trace_start, command_start)) = timing {
        let command_duration = command_start.elapsed();
        // ให้ worker ทำงานต่อได้หากเขียน log ไม่สำเร็จ
        let _ = write_input_trace(
            io::stderr().lock(),
            action,
            target,
            command_start.duration_since(trace_start),
            command_duration,
            &result,
        );
    }
}

/// `at_ms` นับจากเปิดโปรแกรมจนเริ่มเรียกคำสั่ง และ `command_ms` วัดเวลาที่ใช้รอผลของ `xdotool`
/// ใช้ `Instant` เพื่อให้เวลาที่วัดไม่เปลี่ยนตามการปรับนาฬิการะบบ
fn write_input_trace(
    mut output: impl Write,
    action: &str,
    target: &str,
    offset: Duration,
    duration: Duration,
    result: &io::Result<ExitStatus>,
) -> io::Result<()> {
    let outcome = match result {
        Ok(status) if status.success() => "ok".to_owned(),
        Ok(status) => status.to_string(),
        Err(error) => format!("io error: {error}"),
    };
    writeln!(
        output,
        "input action={action:?} target={target:?} at_ms={:.3} command_ms={:.3} result={outcome:?}",
        offset.as_secs_f64() * 1_000.0,
        duration.as_secs_f64() * 1_000.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ใช้เวลาจำลองเพื่อตรวจความละเอียดทศนิยมและการเก็บข้อความ error ไว้ในบรรทัดเดียว
    #[test]
    fn formats_input_timing_and_io_error_on_one_line() {
        let mut output = Vec::new();
        let result = Err(io::Error::new(io::ErrorKind::NotFound, "missing\nxdotool"));

        write_input_trace(
            &mut output,
            "keydown",
            "Shift_L",
            Duration::from_micros(123_456),
            Duration::from_micros(4_567),
            &result,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "input action=\"keydown\" target=\"Shift_L\" at_ms=123.456 command_ms=4.567 result=\"io error: missing\\nxdotool\"\n",
        );
    }
}
