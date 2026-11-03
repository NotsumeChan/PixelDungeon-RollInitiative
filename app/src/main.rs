fn main() {
    env_logger::init();
    app::run(winit::event_loop::EventLoop::new().expect("bucle de eventos"));
}
