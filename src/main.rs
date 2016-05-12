extern crate nalgebra;
extern crate ncollide;
extern crate graphsearch;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use graphsearch::{Graph, Node, Vertex};
use nalgebra::{Vector3, Point3, Identity, Norm, distance};
use ncollide::bounding_volume::BoundingSphere;
use ncollide::query::{Ray, RayCast};

// Earth's radius in km
pub const EARTH_RADIUS: f64 = 6371.0;


fn main() {
    let file_path = Path::new("data_file.txt");
    let (satellites, route) = parse_data_file(file_path);
    let mut locations = Vec::new();
    locations.extend(satellites);
    locations.push(Location {id: "START".to_string(), position: route.start});
    locations.push(Location {id: "END".to_string(), position: route.end});
    let network = generate_line_of_sight_network(&locations);
    let path_of_signal = network.transmit_signal();
    println!("{}", path_of_signal[1..path_of_signal.len()-1].join(","));
}


fn convert_lat_long_to_vector(latitude: f64, longtitude: f64) -> Vector3<f64> {
    let lat = latitude.to_radians();
    let long = longtitude.to_radians();
    let x = EARTH_RADIUS * lat.cos() * long.cos();
    let y = EARTH_RADIUS * lat.cos() * long.sin();
    let z = EARTH_RADIUS * lat.sin();
    Vector3::new(x, y, z)
}

// 2 Vectors have line of sight if the vector created between the 2 doesn't intersect the earth.
pub fn has_line_of_sight(v1: Vector3<f64>, v2: Vector3<f64>) -> bool {
    let earth = BoundingSphere::new(Point3::new(0.0, 0.0, 0.0), EARTH_RADIUS);
    let sphere2 = BoundingSphere::new(v2.to_point(), 0.1);

    let direction = (v2 - v1).normalize();
    let ray = Ray::new(v1.to_point(), direction);

    let earth_toi = earth.toi_with_ray(&Identity::new(), &ray, true);
    let sphere2_toi = sphere2.toi_with_ray(&Identity::new(), &ray, true);

    earth_toi.is_none() || match sphere2_toi {
        Some(toi) => toi < earth_toi.unwrap(),
        None => false
    }
}


fn generate_line_of_sight_network(locations: &[Location]) -> Network {
    let mut graph = Vec::new();
    for s1 in locations.iter() {
        let mut adjacent = Vec::new();
        for (i, s2) in locations.iter().enumerate() {
            if s1 != s2 && has_line_of_sight(s1.position, s2.position) {
                adjacent.push(Vertex{
                    cost: distance(&s1.position.to_point(), &s2.position.to_point()) as i32,
                    node: i
                });
            }
        }
        println!("{:?}: {:?}", s1.id, adjacent);
        graph.push(Node{content: s1.clone(), adjacent: adjacent});
    }
    Network::new(graph)
}


struct Network {
    graph: Graph<Location>
}
impl Network {
    fn new(v: Vec<Node<Location>>) -> Self {
        Network {
            graph: Graph::new(v)
        }
    }

    fn transmit_signal(&self) -> Vec<String> {
        let path = self.graph.search_using_index(20, 21);
        let mut result = Vec::new();
        if let Some(path) = path {
            for i in path {
                let satellite = self.graph.index_to_node(i).unwrap();
                result.push(satellite.content.id.to_owned());
            }
        }
        result
    }
}


#[derive(Clone, Copy, Debug)]
struct Route {
    start: Vector3<f64>,
    end: Vector3<f64>,
}
impl Route {
    fn new(start_lat: f64, start_long: f64, end_lat: f64, end_long: f64) -> Self {
        let start = convert_lat_long_to_vector(start_lat, start_long);
        let end = convert_lat_long_to_vector(end_lat, end_long);
        Route {
            start: start + (0.1 * start.normalize()),
            end: end + (0.1 * end.normalize())
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    id: String,
    position: Vector3<f64>
}
impl Location {
    fn new(id: String, latitude: f64, longtitude: f64, altitude: f64) -> Self {
        let v = convert_lat_long_to_vector(latitude, longtitude);
        Location {
            id: id,
            position: v + (altitude * v.normalize())
        }
    }
}


fn parse_data_file(path: &Path) -> (Vec<Location>, Route) {
    let file_path = Path::new(path);
    let display = path.display();

    let file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => BufReader::new(file)
    };

    let mut satellites = Vec::new();
    let mut route = None;

    for (i, line) in file.lines().enumerate() {
        let s = line.unwrap();
        if i == 0 {
            println!("Parsing file {}", s);
        } else if s.starts_with("ROUTE") {
            let parsed: Vec<f64> = s.split(",").skip(1).map(|s| s.parse().unwrap()).collect();
            route = Some(Route::new(parsed[0], parsed[1], parsed[2], parsed[3]));
        } else {
            let parsed: Vec<&str> = s.split(",").collect();
            satellites.push(Location::new(
                parsed[0].to_string(),
                parsed[1].parse::<f64>().unwrap(),
                parsed[2].parse::<f64>().unwrap(),
                parsed[3].parse::<f64>().unwrap()
            ));
        }
    }

    (satellites, route.unwrap())
}
