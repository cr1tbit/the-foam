use std::sync::atomic::Ordering;

mod body;
mod partition;
mod quadtree;
mod renderer;
mod simulation;
mod utils;

use renderer::Renderer;
use simulation::Simulation;

use std::time::Duration;
use std::time::Instant;

fn sleep_until(time: Instant) {
    let now = Instant::now();
    if now < time {
        std::thread::sleep(time - now);
    }
}

fn main() {
    let config = quarkstrom::Config {
        window_mode: quarkstrom::WindowMode::Windowed(900, 900),
    };

    let mut simulation = Simulation::new();

    std::thread::spawn(move || {
        let max_fps = 60.0;
        let frame_time = Duration::from_secs_f32(1.0/max_fps);
        let mut next_frame = Instant::now();

	    loop {
	        if renderer::PAUSED.load(Ordering::Relaxed) {
	            std::thread::yield_now();
	        } else {
	            simulation.step();
                sleep_until(next_frame);
                next_frame += frame_time;
	        }
	        render(&mut simulation);
	    }
    });

    quarkstrom::run::<Renderer>(config);
}

fn render(simulation: &mut Simulation) {
    let mut lock = renderer::UPDATE_LOCK.lock();
    for body in renderer::SPAWN.lock().drain(..) {
        simulation.bodies.push(body);
    }
    {
        let mut lock = renderer::BODIES.lock();
        lock.clear();
        lock.extend_from_slice(&simulation.bodies);
    }
    {
        let mut lock = renderer::QUADTREE.lock();
        lock.clear();
        lock.extend_from_slice(&simulation.quadtree.nodes);
    }
    *lock |= true;
}
