// This is a simple example to use editor hint macros on code that you don't want running while in the editor.

use godot::{dont_run_in_editor, editor_hint, prelude::*};

struct EditorHints;

#[gdextension]
unsafe impl ExtensionLibrary for EditorHints {}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct IsEditor {
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for IsEditor {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        dont_run_in_editor!();
        godot_print!("Hello, godot 4!")
    }

    fn process(&mut self, delta: f64) {
        if editor_hint!() {
            godot_print!("physics delta in editor: {delta}");
        } else {
            godot_print!("physics delta in runtime: {delta}");
        }
    }
}
