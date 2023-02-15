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

static URI: &str = "http://cascading.reasoning.io/edge/";

static query_str: &str = "Select * WHERE { ?obs a <http://www.w3.org/ns/sosa/Observation>;
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

fn test_observation_10(bench: &mut Bencher){
    let num_offices: usize = 100;
    let num_observations = 10;
    let index = create_data(num_offices, num_observations);
    bench.iter(|| {
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
    });
}
fn test_observation_100(bench: &mut Bencher){
    let num_offices: usize = 100;
    let num_observations = 100;
    let index = create_data(num_offices, num_observations);    bench.iter(|| {
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
    });
}
fn test_observation_1000(bench: &mut Bencher){
    let num_offices: usize = 100;
    let num_observations = 1000;
    let index = create_data(num_offices, num_observations);    bench.iter(|| {
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
    });
}
fn test_observation_10000(bench: &mut Bencher){
    let num_offices: usize = 100;
    let num_observations = 10000;
    let index = create_data(num_offices, num_observations);    bench.iter(|| {
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
    });
}
fn test_rewrite(bench: &mut Bencher){
    let num_offices: usize = 100;
    let num_observations = 10000;
    let index = create_data(num_offices, num_observations);    bench.iter(|| {
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
    });
}
benchmark_group!(benches, test_observation_10, test_observation_100,test_observation_1000,test_observation_10000);
benchmark_main!(benches);