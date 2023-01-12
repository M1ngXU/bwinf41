use std::f64::consts::PI;

struct Location {
    x: f64,
    y: f64,
}

struct Turn {
    from: usize,
    to: usize,
    angle: f64,
}

fn calculate_route(locations: &[Location], turns: &[Turn]) -> Vec<usize> {
    let mut route = Vec::new();
    let mut visited = vec![false; locations.len()];

    // Start at an arbitrary location
    route.push(0);
    visited[0] = true;

    while route.len() < locations.len() {
        let current = *route.last().unwrap();

        let mut next = None;
        let mut next_angle = std::f64::INFINITY;

        // Find the next location to visit that has the smallest angle
        for turn in turns {
            if turn.from == current && !visited[turn.to] && turn.angle <= next_angle {
                next = Some(turn.to);
                next_angle = turn.angle;
            }
        }

        if let Some(next) = next {
            route.push(next);
            visited[next] = true;
        } else {
            // If there are no more locations to visit, terminate the loop
            break;
        }
    }

    route
}

fn main() {
    let locations = vec![
        Location { x: 0.0, y: 0.0 },
        Location { x: 1.0, y: 0.0 },
        Location { x: 1.0, y: 1.0 },
        Location { x: 0.0, y: 1.0 },
    ];

    let turns = vec![
        Turn { from: 0, to: 1, angle: 0.0 },
        Turn { from: 1, to: 2, angle: PI / 2.0 },
        Turn { from: 2, to: 3, angle: PI },
        Turn { from: 3, to: 0, angle: 3.0 * PI / 2.0 },
    ];

    let route = calculate_route(&locations, &turns);

    println!("Route: {:?}", route);
}