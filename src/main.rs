use std::{sync::Arc, thread};
use druid::{Event, EventCtx, Selector, Target, widget::{Button, Controller, Flex, Label, List, Scope, ScopeTransfer}};
use druid::{
    AppLauncher, Data, Env, LocalizedString, Menu, Widget, WidgetExt, WindowDesc, WindowId, Lens,
};
use druid::im::Vector;

/// MyComp is a reusable collection of widget
#[derive(Clone, Data, Lens)]
struct MyCompData {
    id: Arc<usize>,
    titles: Vector<String>
}

impl MyCompData {
    pub fn new(id: Arc<usize>) -> Self {
        Self { 
            id,
            titles: Vector::new(),
        }
    }
}

// External widgets will call build_comp and use a ScopeTransfer
fn build_comp(id: Arc<usize>) -> impl Widget<MyCompData> {
    let compdata = MyCompData::new(Arc::clone(&id));
    let title = Label::new(|d: &MyCompData, _: &Env| format!("{}", d.id));
    let btn = Button::new("ClickMe".to_string())
        .on_click(|ctx, data: &mut MyCompData, _| {
            let event_sink = ctx.get_external_handle();
            let i = Arc::clone(&data.id);
            thread::spawn(move || bg_send_message(event_sink, i));
        });
    let list = List::new(|| {
        Label::new(|d: &String, _: &Env| format!("{}", d))
    }).lens(MyCompData::titles);
    Flex::column()
        .with_child(
            Flex::row()
            .with_child(title)
            .with_child(btn)
        ).with_child(list)
        .controller(compdata)
}

// External data will be transformed to Payload to be dealt on MyCompData
struct Payload {
    my_comp_data_id: Arc<usize>,
    message: String,
}
const SEND_MESSAGE: Selector<Payload> = Selector::new("send-message-mycomp");

// Other threads will call bd_send_message to interact with MyCompData
// with target MyCompData id
fn bg_send_message(event_sink: druid::ExtEventSink, id: Arc<usize>) {
    let payload = Payload {
        my_comp_data_id: id,
        message: "Message".to_string(),
    };
    if event_sink
        .submit_command(SEND_MESSAGE, payload, Target::Auto).is_err()
    {
        println!("Error");
    }
}

trait MyCompDataMessager {
    fn receive_message(&mut self, _payload: &Payload) {}
}

impl MyCompDataMessager for MyCompData {
    fn receive_message(&mut self, payload: &Payload) {
        if self.id == payload.my_comp_data_id {
            let idx = self.titles.len();
            self.titles.push_back(format!("{} {} for MyComp {}", payload.message.clone(), idx, self.id));
        }
    }
}

// Event controller for MyCompData
impl<T: MyCompDataMessager, W: Widget<T>> Controller<T, W> for MyCompData {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(SEND_MESSAGE) => {
                if let Some(message) = cmd.get(SEND_MESSAGE) {
                    data.receive_message(message.clone());
                }
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}


/// main window
#[derive(Clone, Data, Lens)]
pub struct AppData {
}

impl Default for AppData {
    fn default() -> Self {
        Self {
        }
    }
}

// Scope to isolate MyComp
struct MyCompDataTransfer;

impl ScopeTransfer for MyCompDataTransfer {
    type In = AppData;
    type State = MyCompData;
    fn read_input(&self, _state: &mut Self::State, _inner: &Self::In) {}
    fn write_back_input(&self, _state: &Self::State, _inner: &mut Self::In) {}
}

// As MyComp doesn't know who is calling, a wrapper to Scope MyComp to AppData
pub fn build_widget(id: Arc<usize>) -> impl Widget<AppData> {
    let widget = build_comp(Arc::clone(&id));
    Scope::from_function(|_| MyCompData::new(id), MyCompDataTransfer, widget)
}

fn build_app() -> impl Widget<AppData> {
    Flex::column()
        .with_child(
            Flex::row()
            .with_child(
                build_widget(Arc::new(0))
            ).with_child(
                build_widget(Arc::new(1))
            ).with_child(
                build_widget(Arc::new(2))
            )
        )
}


// Menu to fix Cmd+Q bug on mac
#[allow(unused_assignments, unused_mut)]
fn make_menu<T: Data>(_window_id: Option<WindowId>, _app_state: &AppData, _env: &Env) -> Menu<T> {
    let mut base = Menu::empty();
    #[cfg(target_os = "macos")]
    {
        base = base.entry(druid::platform_menus::mac::application::default())
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        base = base.entry(druid::platform_menus::win::file::default());
    }
    base
}

pub fn main() {
    let main_window = WindowDesc::new(build_app())
        .menu(make_menu)
        .title(LocalizedString::new("main-window-title").with_placeholder("MyComp"));
    let data = AppData::default();
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}
