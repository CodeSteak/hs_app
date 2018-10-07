
fn test_key_in() {
    term_setup();

    let mut key_buffer = [0u8; 16];
    let mut key_buffer_filled = 0usize;

    loop {
        /*match step_input() {
            Some(s) => println!("\t\t\t\t{:3} {:2X}", s, s),
            _ => (),
        }*/

        let (k, r, f) = ui::keys::advanced_keys(key_buffer, key_buffer_filled);
        key_buffer = r;
        key_buffer_filled = f;

        println!("Got : {:?}", k);

        if k == ui::keys::Key::EOF {
            break;
        }

        if k == ui::keys::Key::ESC {
            break;
        }

        thread::sleep(Duration::from_millis(100));
        println!("----\n");
    }

    term_unsetup();
}