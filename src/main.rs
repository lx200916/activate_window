use std::process::exit;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use wayland_client::{Connection, Dispatch, event_created_child, EventQueue, protocol::wl_registry, Proxy, QueueHandle};
use clap::Parser;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use crate::wl_registry::Event;
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1;
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1;
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::Event as ZwlrForeignToplevelHandleV1Event;
use anyhow::Result;

struct MyState{
    title: String,
    app_id: String,
    contains: bool,
    running: bool,
    top_manager: Option<ZwlrForeignToplevelManagerV1>,
    seat: Option<WlSeat>,
    list_app_id: bool
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The title of the window to activate. If contains is true, the first window whose title contains this string will be activated.
    #[arg( long,short = 't')]
    title: Option<String>,
    /// The app_id of the window to activate.
    #[arg( long,short = 'a')]
    app_id: Option<String>,
    /// If true, the first window whose title contains the title string will be activated.
    #[arg( long,short,default_value = "false")]
    contains: bool,
    ///List all the windows and their app_id.
    #[arg( long,short='L',default_value = "false")]
    list_app_id: bool,
}
impl TryFrom<Args> for MyState{
    type Error = anyhow::Error;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        if args.title.is_none()&&args.app_id.is_none()&&!args.list_app_id {
            return Err(anyhow::anyhow!("You must specify either a title or an app_id"));
        }
        Ok(MyState{
            title: args.title.unwrap_or(String::new()),
            app_id: args.app_id.unwrap_or(String::new()),
            contains: args.contains,
            running: true,
            top_manager: None,
            seat: None,
            list_app_id: args.list_app_id
        })
    }

}

impl Dispatch<wl_registry::WlRegistry,()> for MyState{
    fn event(state: &mut Self, proxy: &WlRegistry, event: Event, data: &(), conn: &Connection, qhandle: &QueueHandle<Self>) {
        if let Event::Global { name, interface, .. } = event {
            match   &interface[..] {
                "zwlr_foreign_toplevel_manager_v1" =>{
                let top_level =
                    proxy.bind::<ZwlrForeignToplevelManagerV1, _, _>(name, 3, qhandle, ());
                state.top_manager=Some(top_level);
                // println!("Get! zwlr_foreign_toplevel_manager_v1");
            }
                "wl_seat" => {
                    let seat = proxy.bind::<WlSeat, _, _>(name, 8, qhandle, ());
                    state.seat = Some(seat);
                    // println!("Get! wl_seat");
                }
                _=>{}

        }}
    }
}

impl Dispatch<WlSeat,()> for MyState{
    fn event(state: &mut Self, proxy: &WlSeat, event: wayland_client::protocol::wl_seat::Event, data: &(), conn: &Connection, qhandle: &QueueHandle<Self>) {
        // println!("Event WlSeat");
        if let wayland_client::protocol::wl_seat::Event::Name { name } = event {
            // println!("{:?}",name);
        }

    }
}
impl Dispatch<ZwlrForeignToplevelManagerV1,()> for MyState{
    fn event(state: &mut Self, proxy: &ZwlrForeignToplevelManagerV1, event: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event, data: &(), conn: &Connection, qhandle: &QueueHandle<Self>) {
        // println!("Event ZwlrForeignToplevelManagerV1");
        match event  {
            wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                // println!("{:?}",toplevel);
            }
            wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event::Finished => {

                exit(0);
            }
            _ => {}
        }

    }
    event_created_child!(MyState,ZwlrForeignToplevelManagerV1,[
        0=>(ZwlrForeignToplevelHandleV1,())
    ]);


}
impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for MyState{
    fn event(state: &mut Self, proxy: &ZwlrForeignToplevelHandleV1, event: ZwlrForeignToplevelHandleV1Event, data: &(), conn: &Connection, qhandle: &QueueHandle<Self>) {

        match event {
            ZwlrForeignToplevelHandleV1Event::Title { title } => {
                // println!("{:?}",title);
                if state.title==title||(title.contains(state.title.as_str())&&state.contains) {
                    if let Some(seat) = state.seat.as_ref() {
                        proxy.activate(seat);
                        state.top_manager.as_ref().unwrap().stop();
                    }

                }
            }
            ZwlrForeignToplevelHandleV1Event::AppId { app_id } => {
                if state.app_id == app_id{
                    if let Some(seat) = state.seat.as_ref() {
                        proxy.activate(seat);
                        state.top_manager.as_ref().unwrap().stop();
                    }
                }
                if state.list_app_id {
                    println!("{}",app_id);
                }

            }
            _ => {}
        }

    }
}
fn main() {
    let args = Args::parse();
    println!("{:?}",args);
    let conn = Connection::connect_to_env().expect("Failed to connect to the wayland server");
    let display = conn.display();
    let mut event_queue: EventQueue<MyState> = conn.new_event_queue();
    let qhandle = event_queue.handle();
    display.get_registry(&qhandle, ());
    let mut state = MyState::try_from(args).unwrap();
    thread::spawn(move || {
        sleep(Duration::from_secs(2));
        exit(1)
    });
    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

}
