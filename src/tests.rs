//! ทดสอบ worker ด้วยตัวส่ง input ที่เก็บคำสั่งไว้ตรวจสอบในหน่วยความจำ

use super::*;
use std::sync::mpsc;

/// เตรียม worker ในสถานะ `Run` และคืนรายการคำสั่งสำหรับตรวจลำดับการส่ง input
fn recording_controller() -> (ControllerContext, Arc<Mutex<Vec<String>>>) {
    let commands = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&commands);
    let controller = ControllerContext::new(Arc::new(move |action, target| {
        recorded.lock().unwrap().push(format!("{action} {target}"));
    }));
    *controller.worker_request.lock().unwrap() = WorkerRequest::Run(controller.this_worker_run_id);
    (controller, commands)
}

/// ตรวจว่าการแตะปุ่มส่ง `keydown` ก่อน `keyup` และล้างสถานะปุ่มค้างหลังจบ
#[test]
fn tap_sends_keydown_before_keyup() {
    let (controller, commands) = recording_controller();

    assert!(controller.system_tap_key("q"));
    assert_eq!(*commands.lock().unwrap(), ["keydown q", "keyup q"]);
    assert!(controller
        .system_pressed_keyboard_keys
        .lock()
        .unwrap()
        .is_empty());
}

/// ตรวจว่าการหยุด เริ่มรอบใหม่ หรือปิด worker ปล่อยเฉพาะปุ่มที่ยังกดค้าง
#[test]
fn stopping_or_restarting_releases_held_inputs_once() {
    for request in [
        WorkerRequest::Idle,
        WorkerRequest::Run(1),
        WorkerRequest::Shutdown,
    ] {
        let (controller, commands) = recording_controller();
        controller.system_key_down("s");
        controller.system_mouse_button_down(3);
        controller.system_click_mouse_button(1);

        *controller.worker_request.lock().unwrap() = request;
        assert!(!controller.sleep_seconds_interruptible(1.0));
        // เรียก cleanup ซ้ำเพื่อตรวจว่าจะไม่มีคำสั่งปล่อยปุ่มเดิมเพิ่ม
        controller.release_all_system_inputs();

        assert_eq!(
            *commands.lock().unwrap(),
            [
                "keydown s",
                "mousedown 3",
                "click 1",
                "keyup s",
                "mouseup 3"
            ]
        );
    }
}

/// จำลองการกด W ค้างเพื่อตรวจว่า worker ปล่อย S และใช้ flag ข้ามสกิลได้ครั้งเดียว
#[test]
fn manual_pause_releases_held_input_and_skips_once() {
    let (key_released, wait_for_release) = mpsc::channel();
    let controller = ControllerContext::new(Arc::new(move |action, target| {
        if (action, target) == ("keyup", "s") {
            key_released.send(()).unwrap();
        }
    }));
    *controller.worker_request.lock().unwrap() = WorkerRequest::Run(controller.this_worker_run_id);
    controller.system_key_down("s");

    let pause_keys = &controller.user_pressed_pause_keyboard_keys;
    pause_keys.lock().unwrap().insert(Key::KeyW);
    thread::scope(|scope| {
        let simulate_user = scope.spawn(move || {
            // จำลองการปล่อย W หลัง worker ปล่อย S และใช้ timeout ให้ test จบได้หากไม่พบคำสั่ง
            let released = wait_for_release.recv_timeout(Duration::from_secs(2));
            pause_keys.lock().unwrap().remove(&Key::KeyW);
            released.expect("worker should release the key while paused");
        });

        assert!(controller.wait_until_user_is_idle_for_seconds(0.0));
        simulate_user.join().unwrap();
    });

    assert!(controller.take_should_skip_next_passive_skill_once());
    assert!(!controller.take_should_skip_next_passive_skill_once());
    assert!(controller.take_should_skip_rest_of_current_combo_once());
    assert!(!controller.take_should_skip_rest_of_current_combo_once());
}
