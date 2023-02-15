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

static EVENT_BLUEPRINT: &str = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";


fn test_static_offices_10_edge(bench: &mut Bencher){
    let num_offices: usize = 10;
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec());
    // rewrite for edge processing
    let query = Query::parse(QUERY_STR, None).unwrap();
    let rewritten_query = rewrite_for_edge(static_triples, &EVENT_BLUEPRINT, &query);

    let index = create_data_edge();

    bench.iter(|| {
        let plan = eval_query(&rewritten_query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_100_edge(bench: &mut Bencher){
    let num_offices: usize = 100;
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec());
    // rewrite for edge processing
    let query = Query::parse(QUERY_STR, None).unwrap();
    let rewritten_query = rewrite_for_edge(static_triples, &EVENT_BLUEPRINT, &query);

    let index = create_data_edge();

    bench.iter(|| {
        let plan = eval_query(&rewritten_query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_1000_edge(bench: &mut Bencher){
    let num_offices: usize = 1000;
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec());
    // rewrite for edge processing
    let query = Query::parse(QUERY_STR, None).unwrap();
    let rewritten_query = rewrite_for_edge(static_triples, &EVENT_BLUEPRINT, &query);

    let index = create_data_edge();

    bench.iter(|| {
        let plan = eval_query(&rewritten_query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}
fn test_static_offices_10000_edge(bench: &mut Bencher){
    let num_offices: usize = 10000;
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec());
    // rewrite for edge processing
    let query = Query::parse(QUERY_STR, None).unwrap();
    let rewritten_query = rewrite_for_edge(static_triples, &EVENT_BLUEPRINT, &query);

    let index = create_data_edge();

    bench.iter(|| {
        let plan = eval_query(&rewritten_query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
    });
}

fn create_data_edge() -> TripleIndex {
    let observations = generate_observation("obs1", "0_office1", "100", None);
    let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
    let mut index = TripleIndex::new();
    observation_triples.into_iter().for_each(|t| index.add(t));
    index
}
benchmark_group!(benches, test_static_offices_10_edge, test_static_offices_100_edge, test_static_offices_1000_edge, test_static_offices_10000_edge);
benchmark_main!(benches);