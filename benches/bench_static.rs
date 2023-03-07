#[macro_use]
extern crate bencher;

use std::collections::HashSet;
use bencher::Bencher;
use roxi::parser::{Parser, Syntax};
use roxi::sparql::{Binding, eval_query, evaluate_plan_and_debug, PlanNode};
use roxi::tripleindex::TripleIndex;
use spargebra::Query;
use roxi::triples::Triple;
use edge_rewrite::generator::{generate_building, generate_observation};
use edge_rewrite::rewrite::rewrite_for_edge;

static URI: &str = "http://cascading.reasoning.io/edge/";

static QUERY_STR: &str = "Select * WHERE { ?obs a <http://www.w3.org/ns/sosa/Observation>;
<http://www.w3.org/ns/sosa/hasSimpleResult> ?value;
<http://www.w3.org/ns/sosa/madeBySensor> ?sensor.
?sensor <http://cascading.reasoning.io/edge/hasLocation> ?loc.
?loc <http://cascading.reasoning.io/edge/hasName> \"office1\".
 }";



fn create_data(num_offices: usize, num_observations: usize) -> TripleIndex {
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

    let static_triples = generate_building(num_offices, properties.to_vec());


    let mut index = TripleIndex::new();
    static_triples.into_iter().for_each(|t| index.add(t));
    for obs_counter in 0..num_observations {
        let observations = generate_observation(format!("obs{}", obs_counter).as_str(),
                                                "0_office1",
                                                "100",
                                                None);
        let observation_triples = Parser::parse_triples( & observations,
                                                         Syntax::NQuads).unwrap();
        observation_triples.into_iter().for_each( | t| index.add(t));
    }


    index
}

fn create_data_static_increase(num_offices: usize) -> TripleIndex {
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

    let static_triples = generate_building(num_offices, properties.to_vec());

    let observations = generate_observation("obs1", "0_office1", "100", None);
    let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
    let mut index = TripleIndex::new();
    static_triples.into_iter().for_each(|t| index.add(t));
    observation_triples.into_iter().for_each(|t| index.add(t));


    index
}

fn test_static_offices_10(bench: &mut Bencher){
    let num_offices: usize = 10;
    let index = create_data_static_increase(num_offices);
    let query = Query::parse(QUERY_STR, None).unwrap();

    bench.iter(|| {
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_100(bench: &mut Bencher){
    let num_offices: usize = 100;
    let index = create_data_static_increase(num_offices);
    let query = Query::parse(QUERY_STR, None).unwrap();

    bench.iter(|| {
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_1000(bench: &mut Bencher){
    let num_offices: usize = 1000;
    let index = create_data_static_increase(num_offices);
    let query = Query::parse(QUERY_STR, None).unwrap();

    bench.iter(|| {
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_10000(bench: &mut Bencher){
    let num_offices: usize = 10000;
    let index = create_data_static_increase(num_offices);
    let query = Query::parse(QUERY_STR, None).unwrap();

    bench.iter(|| {
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
benchmark_group!(benches, test_static_offices_10,test_static_offices_100,test_static_offices_1000, test_static_offices_10000);
benchmark_main!(benches);
