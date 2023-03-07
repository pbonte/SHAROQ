use std::collections::HashSet;
use std::fs;
use roxi::parser::{Parser, Syntax};
use roxi::sparql::{Binding, eval_query, evaluate_plan_and_debug};
use roxi::tripleindex::TripleIndex;
use roxi::triples::{Rule, Triple};
use roxi::TripleStore;
use spargebra::Query;
use crate::generator::{generate_building, generate_observation};
use crate::rewrite::{rewrite_for_edge, rewrite_query, rewrite_rules, rewrite_rules_for_edge};

mod generator;
mod rewrite;


static URI: &str = "http://cascading.reasoning.io/edge/";
static NUM_TEST: usize = 30;
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

fn create_data_sensor_increase(num_offices: usize, num_observations: usize) -> TripleIndex {
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

    let static_triples = generate_building(num_offices, properties.to_vec(),Some("<http://static/>".to_string()));


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

    let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

    let observations = generate_observation("obs1", "0_office1", "100", None);
    let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
    let mut index = TripleIndex::new();
    static_triples.into_iter().for_each(|t| index.add(t));
    observation_triples.into_iter().for_each(|t| index.add(t));


    index
}
fn create_data_edge() -> TripleIndex {
    let observations = generate_observation("obs1", "0_office1", "100", None);
    let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
    let mut index = TripleIndex::new();
    observation_triples.into_iter().for_each(|t| index.add(t));
    index
}

fn average(numbers: &Vec<u128>) -> f64 {
    numbers.iter().sum::<u128>() as f64 / numbers.len() as f64
}
fn main() {
    println!("Static Data increase:");
    let office_sizes = [10, 100, 1000, 10000];
    for size in office_sizes{
        let avg = static_data_increase(size);
        println!("Size: {:?},\t\t AVG: {:.2?}", size, avg);
    }
    println!("Sensor Data increase:");
    let observation_sizes = [10, 100, 1000, 10000];
    for size in observation_sizes{
        let avg = sensor_data_increase(100, size);
        println!("Size: {:?},\t\t AVG: {:.2?}", size, avg);
    }
    println!("Edge evaluation with static data increase:");
    for size in office_sizes{
        let avg = edge_data(size);
        println!("Size: {:?},\t\t AVG: {:.2?}", size, avg);
    }
    println!("Hierarchy increase:");
    let hierarchies = [10, 100, 1000, 10000];
    for size in hierarchies{
        let avg =     reasoning(1,size);
        println!("Size: {:?},\t\t AVG: {:.2?}", size, avg);
    }
    println!("REasoning edge:");
    let hierarchies = [10, 100, 1000];
    for size in hierarchies{
        let avg =     reasoning_edge(1,size);
        println!("Size: {:?},\t\t AVG: {:.2?}", size, avg);
    }

}
fn rewrite_registered_rules(static_triples: Vec<Triple>, rules:String, backward_head: Triple ) -> Vec<Rule> {
    let mut store = TripleStore::new();
    static_triples.into_iter().for_each(|t|store.add(t));
    store.load_rules(&*rules);

    //load the event shape
    store.load_triples(EVENT_BLUEPRINT,Syntax::NQuads);


    let mut new_rules = rewrite_rules_for_edge(&store.triple_index, &store.rules_index, &backward_head);
    new_rules
}
fn reasoning_edge(num_offices: usize, hierarchy_size: usize) -> f64 {
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

    let rules = create_hierarchy(hierarchy_size);
    let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://cascading.reasoning.io/edge/ComfortObservation>".to_string());

    let rewritten_rules = rewrite_registered_rules(static_triples.clone(), rules, backward_head);
    //println!("Rewritten rules {:?}",TripleStore::decode_rules(&*rewritten_rules));

    let mut aveges = Vec::with_capacity(NUM_TEST);

    for test_it in 0..NUM_TEST {

        let mut store = TripleStore::new();
        store.load_rules(&*TripleStore::decode_rules(&*rewritten_rules));
        // load observation
        let observations = generate_observation("obs1", "2_office0", "100", None);
        let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
        observation_triples.into_iter().for_each(|t| store.add(t));

        use std::time::Instant;
        let now = Instant::now();
        // let before = store.len();
        store.materialize();
        // let after = store.len();
        let elapsed = now.elapsed();
        // println!("before {:?}, after{:?}", before,after);
        aveges.push(elapsed.as_nanos());
    }
    average(&aveges)

}
fn reasoning(num_offices: usize, hierarchy_size: usize) -> f64 {
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

    let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

    let rules = create_hierarchy(hierarchy_size);


    let mut aveges = Vec::with_capacity(NUM_TEST);

    for test_it in 0..NUM_TEST {

        let mut store = TripleStore::new();
        static_triples.clone().into_iter().for_each(|t|store.add(t));
        store.load_rules(&*rules);
        // load observation
        let observations = generate_observation("obs1", "2_office0", "100", None);
        let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
        observation_triples.into_iter().for_each(|t| store.add(t));

        use std::time::Instant;
        let now = Instant::now();
        //let before = store.len();
        store.materialize();
        //let after = store.len();
        let elapsed = now.elapsed();

        aveges.push(elapsed.as_nanos());
    }
    average(&aveges)

}

fn create_hierarchy(hierarchy_size: usize) -> String {
    let mut rules = "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
@prefix : <http://cascading.reasoning.io/edge/>.
@prefix sosa: <http://www.w3.org/ns/sosa/>.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :LoudnessSensor.} => {?x a :LoudnessObservation0.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Loudness.} => {?s a :LoudnessSensor.}.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :TempSensor.} => {?x a :TempObservation.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Temperature.} => {?s a :TempSensor.}.
{?x a :TempObservation} => {?x a :ComfortObservation}.".to_owned();
    for i in 0..hierarchy_size {
        rules.push_str(&format!("\n{{?x a :LoudnessObservation{:?}}} => {{?x a :LoudnessObservation{:?}}}.", i, i + 1));
    }
    rules.push_str(&format!("\n{{?x a :LoudnessObservation{:?}}} => {{?x a :ComfortObservation}}.", hierarchy_size));

    rules
}

fn edge_data(num_offices: usize) -> f64{
    let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
    let static_triples = generate_building(num_offices, properties.to_vec(),Some("<http://static/>".to_string()));
    // rewrite for edge processing
    let query = Query::parse(QUERY_STR, None).unwrap();
    let rewritten_query = rewrite_for_edge(static_triples, &EVENT_BLUEPRINT, &query);

    let index = create_data_edge();
    let mut aveges = Vec::with_capacity(NUM_TEST);

    for test_it in 0..NUM_TEST {
        use std::time::Instant;
        let now = Instant::now();
        let plan = eval_query(&rewritten_query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != 1 {
            println!("Error!");
        }
        let elapsed = now.elapsed();

        aveges.push(elapsed.as_nanos());
    }
    average(&aveges)
}
fn sensor_data_increase(num_offices: usize, num_observations: usize)-> f64{

    let index = create_data_sensor_increase(num_offices, num_observations);
    let mut aveges = Vec::with_capacity(NUM_TEST);
    let query = Query::parse(QUERY_STR, None).unwrap();

    for test_it in 0..NUM_TEST {
        use std::time::Instant;
        let now = Instant::now();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len() ;
        if results != num_observations{
            println!("Error!");
        }
        let elapsed = now.elapsed();

        aveges.push(elapsed.as_nanos());
    }
    average(&aveges)
}
fn static_data_increase(num_office: usize)-> f64{
    let index = create_data_static_increase(num_office);
    let query = Query::parse(QUERY_STR, None).unwrap();
    let mut aveges = Vec::with_capacity(NUM_TEST);
    for test_it in 0..NUM_TEST {
        use std::time::Instant;
        let now = Instant::now();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        let results = iterator.collect::<HashSet<Vec<Binding>>>().len();
        if results != 1 {
            println!("Error!");
        }
        let elapsed = now.elapsed();

        aveges.push(elapsed.as_nanos());
    }
    average(&aveges)
}

fn building_exec(){
    let triples = "<http://www.some.com/sensor1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Sensor> <http://static/>.
<http://www.some.com/sensor1> <http://www.some.com/observes> <http://www.some.com/temp> <http://static/>.
<http://www.some.com/sensor2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Sensor> <http://static/>.
<http://www.some.com/sensor2> <http://www.some.com/observes> <http://www.some.com/humidity> <http://static/>.
<http://www.some.com/temp> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Temp> <http://static/>.
<http://www.some.com/co2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/CO2> <http://static/>.
<http://www.some.com/humidity> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Humidity> <http://static/>.
<http://www.some.com/obs> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/madeBySensor> <http://www.some.com/sensor1> <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/hasValue> \"10\" <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/observedProperty> <http://www.some.com/temp> <http://stream/>.
";

    let rules ="@prefix : <http://www.some.com/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?x rdf:type :Observation. ?x :madeBySensor ?y. ?y rdf:type :TempSensor}=>{?x rdf:type :TempObservation.}
{?x rdf:type :Sensor. ?x :observes ?y. ?y rdf:type :Temp}=>{?x rdf:type :TempSensor.}.
{?x rdf:type :TempObservation} => {?x rdf:type :EnvironmentObservation.}.
";

    let mut store = TripleStore::new();

    let load_warning = store.load_triples(triples, Syntax::NQuads);
    store.load_rules(rules);
    //println!("Encoded: {:?}", store.encoder);
//backward head
    let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://www.some.com/TempObservation>".to_string());

    let mut new_rules = Vec::new();
    let  bindings = rewrite_rules(&store.triple_index, &store.rules_index, &backward_head,  None, &mut new_rules);
    // let bindings = store.triple_index.query(&backward_head, None);
    println!("bindings {:?}",bindings);
    println!("new_rules {:?}",TripleStore::decode_rules(&new_rules));
    assert_eq!("{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation>.\n?x <http://www.some.com/madeBySensor> <http://www.some.com/sensor1>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/TempObservation>.\n}.\n", TripleStore::decode_rules(&new_rules));

    let inferrred = store.materialize();
    println!("Inferred {:?}", TripleStore::decode_triples(&inferrred));


    // rewrite query only
    let triples = "<http://www.some.com/sensor1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Sensor> <http://static/>.
<http://www.some.com/sensor1> <http://www.some.com/observes> <http://www.some.com/temp> <http://static/>.
<http://www.some.com/sensor1> <http://www.some.com/hasLocation> <http://www.some.com/someLocation> <http://static/>.
<http://www.some.com/someLocation> <http://www.some.com/hasName> \"200.009\" <http://static/>.
<http://www.some.com/temp> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Temp> <http://static/>.
<http://www.some.com/co2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/CO2> <http://static/>.
<http://www.some.com/humidity> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Humidity> <http://static/>.
<http://www.some.com/obs> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/madeBySensor> <http://www.some.com/sensor1> <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/hasValue> \"10\" <http://stream/>.
<http://www.some.com/obs> <http://www.some.com/observedProperty> <http://www.some.com/temp> <http://stream/>.
";

    let query ="@prefix : <http://www.some.com/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?x rdf:type :Observation. ?x :madeBySensor ?y. ?y :hasLocation ?loc. ?loc :hasName :someName}=>{?x rdf:type :TempObservation.}
";

    let mut store = TripleStore::new();

    let load_warning = store.load_triples(triples, Syntax::NQuads);
    store.load_rules(query);
    let query = &store.rules_index.rules.get(0).unwrap().body;
    let rewritten_query = rewrite_query(&store.triple_index,query);

    println!("rewritten query {:?}", TripleStore::decode_triples(&rewritten_query));
    assert_eq!("?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation>.\n?x <http://www.some.com/madeBySensor> <http://www.some.com/sensor1>.\n", TripleStore::decode_triples(&rewritten_query));


    // rewrite query with observation template (containing blank nodes)
    let triples = "<http://www.some.com/sensor1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Sensor> <http://static/>.
<http://www.some.com/sensor1> <http://www.some.com/observes> <http://www.some.com/temp> <http://static/>.
<http://www.some.com/sensor1> <http://www.some.com/hasLocation> <http://www.some.com/someLocation> <http://static/>.
<http://www.some.com/someLocation> <http://www.some.com/hasName> \"200.009\" <http://static/>.
<http://www.some.com/temp> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Temp> <http://static/>.
<http://www.some.com/co2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/CO2> <http://static/>.
<http://www.some.com/humidity> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Humidity> <http://static/>.
_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.some.com/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.some.com/hasValue> _:b2 <http://stream/>.
_:b0 <http://www.some.com/observedProperty> _:b3 <http://stream/>.
";

    let query ="@prefix : <http://www.some.com/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?x rdf:type :Observation. ?x :madeBySensor ?y. ?y :hasLocation ?loc. ?loc :hasName :someName}=>{?x rdf:type :TempObservation.}
";

    let mut store = TripleStore::new();

    let load_warning = store.load_triples(triples, Syntax::NQuads);
    store.load_rules(query);
    let query = &store.rules_index.rules.get(0).unwrap().body;
    let rewritten_query = rewrite_query(&store.triple_index,query);
    println!("rewritten query {:?}", rewritten_query);

    println!("rewritten query {:?}", TripleStore::decode_triples(&rewritten_query));
    assert_eq!("x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation>.\nx <http://www.some.com/madeBySensor> <http://www.some.com/sensor1>.\n", TripleStore::decode_triples(&rewritten_query));


    let aaabim_query = "PREFIX math:<http://www.w3.org/2005/xpath-functions/math#>
PREFIX : <http://massif/results/>
PREFIX obelisk: <https://obelisk.ilabt.imec.be/ontology/>
PREFIX sosa: <http://www.w3.org/ns/sosa/>
Select *
WHERE {
            ?co2_obs sosa:observedProperty [a obelisk:CO2AirQuality];
                sosa:madeBySensor ?sensor;
                sosa:hasSimpleResult ?co2_res.

            ?temperature_obs sosa:observedProperty [a obelisk:Temperature];
                sosa:madeBySensor ?sensor;
                sosa:hasSimpleResult ?temperature_res.

            ?rel_humidity_obs sosa:observedProperty [a obelisk:RelativeHumidity];
                sosa:madeBySensor ?sensor;
                sosa:hasSimpleResult ?rel_humidity_res.

            ?sensor obelisk:hasLocation ?loc.
            ?loc <https://w3id.org/props#reference_simple> ?loc_name.

            ?sensor obelisk:thingId ?sensor_label.

 }";
    let aaabim_query = "@prefix : <http://massif/results/>.
@prefix obelisk: <https://obelisk.ilabt.imec.be/ontology/>.
@prefix sosa: <http://www.w3.org/ns/sosa/>.
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
{?sensor obelisk:hasLocation ?loc. ?co2Obs sosa:observedProperty ?co2Type. ?co2Type rdf:type obelisk:CO2AirQuality. ?co2Obs sosa:madeBySensor ?sensor. ?co2Obs sosa:hasSimpleResult ?co2Res. }=>{?co2Obs rdf:type :Co2Observation.}
";

    let mut store = TripleStore::new();
    let triples = fs::read_to_string("files/homelabv3.ttl").unwrap();
    let load_warning = store.load_triples(triples.as_str(), Syntax::Turtle);
    let event = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";
    let load_warning = store.load_triples(event, Syntax::NQuads);

    store.load_rules(aaabim_query);
    let query = &store.rules_index.rules.get(0).unwrap().body;
    let rewritten_query = rewrite_query(&store.triple_index,query);
    println!("rewritten query {:?}", rewritten_query);

    println!("rewritten query {:?}", TripleStore::decode_triples(&rewritten_query));
    assert_eq!("x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation>.\nx <http://www.some.com/madeBySensor> <http://www.some.com/sensor1>.\n", TripleStore::decode_triples(&rewritten_query));

}