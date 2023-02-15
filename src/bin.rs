use std::fs;
use roxi::parser::Syntax;
use roxi::triples::Triple;
use roxi::TripleStore;
use crate::rewrite::{rewrite_query, rewrite_rules};

mod generator;
mod rewrite;




fn main() {

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
