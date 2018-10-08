
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


mod test {

    pub(crate) fn test_render(_state: isize) {
        use ui::termutil::*;
        use ui::*;

        let (w, h) = query_terminal_size_and_reset().unwrap_or((100, 100));

        let mut root = GridV::new()
            .add(
                GridH::new()
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("1")))
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("2")))
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("3"))),
            ).add(
            GridH::new()
                .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("4")))
                .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("5/6"))),
        ).add(GridH::new().add(VBox(
            SIMPLE_BOX,
            Color::BrightYellow,
            VText::simple("Full"),
        )));

        root.try_set_size(w as isize, h as isize);
        root.render_to_stdout();
    }

    pub(crate) fn test_render2(state: &isize) {
        use ui::termutil::*;
        use ui::*;

        let (w, h) = query_terminal_size_and_reset().unwrap_or((20, 20));

        let mut root = GridH::new()
            .add(VBox(
                DOUBLE_BORDER_BOX,
                Color::BrightYellow,
                Spacer::new(VText::simple("Hello World")),
            )).add(VBox(
            SIMPLE_BOX,
            Color::BrightYellow,
            Spacer::new(VText::simple(
                "Hello World. And also: 'Hello Humanity'. And Stuff... This is Filler Text",
            )),
        )).add(VBox(
            BORDER_BOX,
            Color::BrightYellow,
            Margin(
                (4, 2),
                Backgound(
                    Color::BrightBlue,
                    Spacer::new(VText::simple(
                        "\t1\t2\t3\t4\t5\t6\t7\t8\t9\t\n\ntab stops are working!",
                    )),
                ),
            ),
        )).add(VText::simple(&format!(
            "Tic: {:.2}",
            *state as f32 / 1000f32
        )));

        root.try_set_size(w as isize, h as isize);
        root.render_to_stdout();
    }
}
