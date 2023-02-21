use roxi::TripleStore;
#[cfg(not(test))]
use log::{info, warn,trace}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn, println as trace};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;
use roxi::backwardchaining::BackwardChainer;
use roxi::bindings::Binding;
use roxi::encoding::Encoder;
use roxi::parser::{Parser, Syntax};
use roxi::reasoner::Reasoner;
use roxi::ruleindex::RuleIndex;
use roxi::sparql::{eval_query, PlanNode};
use roxi::tripleindex::TripleIndex;
use roxi::triples::{Rule, Triple, VarOrTerm};
use spargebra::{ParseError, Query};

enum SourceType{
    Stream,
    Static
}
impl SourceType{
    fn as_str(&self)->&'static str{
        match self{
            SourceType::Stream => "<http://stream/>",
            SourceType::Static => "<http://static/>"
        }
    }
}
fn get_vars(triple: &Triple) -> Vec<usize>{
    let mut vars = Vec::new();
    if triple.s.is_var(){
        vars.push(triple.s.to_encoded());
    }
    if triple.p.is_var(){
        vars.push(triple.s.to_encoded());
    }
    if triple.o.is_var(){
        vars.push(triple.s.to_encoded());
    }
    vars
}
/// Rewrites a query and inject static bindings in streaming part of the query
pub fn rewrite_query(triple_index: &TripleIndex, query: &[Triple]) -> Vec<Triple> {
    let mut rule_bindings = Binding::new();
    for (i,rule_atom) in query.iter().enumerate() {
        let mut rule_atom = rule_atom.clone();
        let graph_var = VarOrTerm::new_var(format!("?graph_name_{}", i));
        rule_atom.g = Some(graph_var.clone());

        if let Some(mut result_bindings) = triple_index.query(&rule_atom, None) {
            //check if triple is part of the stream
            let encoded_stream_graph = Encoder::add(SourceType::Stream.as_str().to_string());
            let encoded_static_graph = Encoder::add(SourceType::Static.as_str().to_string());

            let mut found = false;
            if let Some(bindings) = result_bindings.get(&graph_var.to_encoded()){
                for b in bindings{
                    if *b == encoded_stream_graph{
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                if result_bindings.get(&graph_var.to_encoded()).is_none(){
                    let mut binding = Binding::new();
                    binding.add(&graph_var.to_encoded(), encoded_static_graph);
                    result_bindings.combine(binding);
                }
                rule_bindings = rule_bindings.join(&result_bindings);


            }else{
                let mut binding = Binding::new();
                binding.add(&graph_var.to_encoded(), encoded_stream_graph);
                rule_bindings = rule_bindings.join(&binding);

            }
        }
    }
    let (bindings_mapping, rewritten_rules) = rewrite_body(query, &mut rule_bindings);
    rewritten_rules
}
/// Rewrites rules, starting from a certain rule head using backward chaining
pub fn rewrite_rules(triple_index: &TripleIndex, rule_index: &RuleIndex, rule_head: &Triple, orig_position: Option<usize>, rewritten_rules: &mut Vec<Rule>) -> Binding {

    let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = BackwardChainer::find_subrules(rule_index, rule_head);

    let mut all_bindings = Binding::new();
    for (sub_rule, var_subs) in sub_rules.into_iter() {
        let mut rule_bindings = Binding::new();
        for (i,rule_atom) in sub_rule.body.iter().enumerate() {
            let mut rule_atom = rule_atom.clone();
            rule_atom.g = Some(VarOrTerm::new_var(format!("?graph_name_{}", i)));

            if let Some(mut result_bindings) = triple_index.query(&rule_atom, None) {
                // println!("Joing left: {:?}", rule_bindings.decode());
                // println!("Joing right: {:?}", result_bindings.decode());
                //remove blank nodes from bindings
                rule_bindings = rule_bindings.join_with_blank_override(&result_bindings, true);
                // println!("Joing result: {:?}", rule_bindings.decode());

            }
            //recursive call
            let recursive_bindings = rewrite_rules(triple_index, rule_index, &rule_atom,  Some(i), rewritten_rules);
            rule_bindings.combine(recursive_bindings);


        }



        let (bindings_mapping, mut rewritten_rules_temp) = rewrite_rule(sub_rule, &mut rule_bindings);

        rewritten_rules.append(&mut rewritten_rules_temp);
        //rename variables
        let mut renamed = rule_bindings.rename(var_subs.clone());
        // add head graph
        if let Some(pos) = orig_position {
            let result = var_subs.iter().map(|(old,new)|bindings_mapping.get(old).unwrap().clone()).reduce(|a,b| a && b).unwrap();
            if result {
                renamed.add(&Encoder::add(format!("?graph_name_{}", pos)), Encoder::add(SourceType::Stream.as_str().to_string()));
            }else{
                renamed.add(&Encoder::add(format!("?graph_name_{}", pos)), Encoder::add(SourceType::Static.as_str().to_string()));
            }
        }
        all_bindings.combine(renamed);
    }
    all_bindings
}



/// rewrites a body of a rule or a query
fn rewrite_body(sub_rule: &[Triple], rule_bindings: &mut Binding) -> (HashMap<usize, bool>,Vec<Triple>) {
//check if head bindings are static or streaming
    let mut bindings_mapping = HashMap::new();
    for (i, triple) in sub_rule.iter().enumerate() {
        let triple_graph = Encoder::add(format!("?graph_name_{}", i));
        let vars = get_vars(&triple);
        for triple_var in vars {
            if let Some(graph_bindings) = rule_bindings.get(&triple_graph) {
                if graph_bindings.contains(&Encoder::add(SourceType::Stream.as_str().to_string())) {
                    if let Some(false) = bindings_mapping.get(&triple_var) {
                        //is already static
                    } else {
                        bindings_mapping.insert(triple_var, true);
                    }
                } else {
                    bindings_mapping.insert(triple_var, false);
                }
            }
        }
    }
    // define which triples in rule where static
    let mut new_body = Vec::new();
    let mut static_binding_names = HashSet::new();


    for (i, triple) in sub_rule.iter().enumerate() {
        let triple_graph = Encoder::add(format!("?graph_name_{}", i));
        if let Some(graph_bindings) = rule_bindings.get(&triple_graph) {
            if graph_bindings.contains(&Encoder::add(SourceType::Stream.as_str().to_string())) {
                new_body.push(triple.clone());
            } else {
                get_vars(triple).into_iter().for_each(|t| {
                    static_binding_names.insert(t);
                    ()
                });
            }
        }
    }
    // substitute static bindings
    let mut bindings_static = rule_bindings.clone();
    let collected_names: Vec<usize> = static_binding_names.into_iter().collect();
    bindings_static.retain_vars(&collected_names);
    if bindings_static.len() == 0{
        //stream only rule
        bindings_static = rule_bindings.clone();
    }
    let mut rewrite_body = Vec::new();


    let bindings_static = remove_blink_nodes_from_binding(&bindings_static);
    for triple in new_body.iter() {
        let new_triples = Reasoner::substitute_triple_with_bindings(triple, &bindings_static);
        // println!("   new triples {:?}", TripleStore::decode_triples(&new_triples));

        rewrite_body.push(new_triples.get(0).unwrap().clone()); //todo account for multiple results
    }

    (bindings_mapping,rewrite_body)
}
/// rewrites one specific rule
fn rewrite_rule( sub_rule: Rc<Rule>, rule_bindings: &mut Binding) -> (HashMap<usize, bool>,Vec<Rule>) {
    let mut rewritten_rules = Vec::new();
    let (bindings_mapping,rewrite_body) = rewrite_body( &sub_rule.body,rule_bindings);
    if !rewrite_body.is_empty() {
        rewritten_rules.push(Rule { head: sub_rule.head.clone(), body: rewrite_body });
    }
    (bindings_mapping,rewritten_rules)
}

fn query_to_triples(query: &Query) -> Vec<Triple> {
    let plan_node = eval_query(query, &TripleIndex::new());
    let mut triples = Vec::new();
    match plan_node{

        PlanNode::Project {child,mapping}=> {
            extract_TP(&mut triples, child);
        }
        _ => {}
    }
    triples
}
fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}
fn triples_to_query(triples: &Vec<Triple>) -> Result<Query, ParseError> {
    let decoded_triples =TripleStore::decode_triples(triples);

    let query_string = format!("Select * WHERE{{ {} }}", decoded_triples);
    Query::parse(query_string.as_str(), None)
}
fn extract_TP(triples: &mut Vec<Triple>, child: Box<PlanNode>) {
    match *child {
        PlanNode::QuadPattern { pattern: triple } => {
            triples.push(triple);
        },
        PlanNode::Join { left, right } => {
            extract_TP(triples, left);
            extract_TP(triples, right);
        }
        _ => {}
    }
}

pub fn rewrite_for_edge(static_triples: Vec<Triple>, event: &str, query: &Query) -> Query {
    // convert query to triples
    let extracted_query = query_to_triples(&query);

    // load static data
    let mut index = TripleIndex::new();
    static_triples.into_iter().for_each(|t| index.add(t));

    //load event shape
    let event_triples = Parser::parse_triples(&event, Syntax::NQuads).unwrap();
    event_triples.into_iter().for_each(|t| index.add(t));

    // rewrite query
    let rewritten_query = rewrite_query(&index, &*extracted_query);
    let rewritten_query = triples_to_query(&rewritten_query).unwrap();
    rewritten_query
}
fn remove_blink_nodes_from_binding(binding: &Binding) -> Binding {
    let mut filtered_binding = Binding::new();
    for key in binding.vars(){
        for val in binding.get(key).unwrap(){
            if let Some(blank) = Encoder::decode(val){
                if !blank.starts_with("_:"){
                    filtered_binding.add(key, val.clone());
                }
            }
        }
    }
    filtered_binding
}

#[cfg(test)]
mod test{
    use roxi::parser::Parser;
    use spargebra::Query;
    use crate::generator::{generate_building, generate_observation};
    use super::*;

    #[test]
    fn test_remove_blank_nodes(){
        let mut binding = Binding::new();
        let encoded_var = Encoder::add("?s".to_string());
        let encoded_val1 = Encoder::add("<http://test>".to_string());
        let encoded_blank_node = Encoder::add("_:b1".to_string());

        binding.add(&encoded_var, encoded_val1);
        binding.add(&encoded_var, encoded_blank_node);
        let no_blanks_binding = remove_blink_nodes_from_binding(&binding);

        let mut expected_binding = Binding::new();
        expected_binding.add(&encoded_var, encoded_val1);

        assert_eq!(expected_binding, no_blanks_binding);
    }



    #[test]
    fn test_rewrite() {
        // generate static data
        let num_offices: usize = 2;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

        let static_triples = generate_building(num_offices, properties.to_vec(), None);
        // define the query
        let query = "Select * WHERE{ ?obs a <http://www.w3.org/ns/sosa/Observation>;
        <http://www.w3.org/ns/sosa/hasSimpleResult> ?value;
        <http://www.w3.org/ns/sosa/madeBySensor> ?sensor.
        ?sensor <http://cascading.reasoning.io/edge/hasLocation> ?loc.
        ?loc <http://cascading.reasoning.io/edge/hasName> \"office1\".}";

        // define event shape
        let event = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";
        let query = Query::parse(query, None).unwrap();

        // rewrite for edge processing
        let rewritten_query = rewrite_for_edge(static_triples, &event, &query);

        let expected_rewritten_query = "Select * WHERE{?obs <http://www.w3.org/ns/sosa/hasSimpleResult> ?value.
?obs <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_0_office1>.
}";
        assert_eq!(Query::parse(expected_rewritten_query, None).unwrap(), rewritten_query);
    }

    #[test]
    fn test_rewrite_with_reasoning(){
        // generate static data
        let num_offices: usize = 1;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

        let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

        let rules ="@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
@prefix : <http://cascading.reasoning.io/edge/>.
@prefix sosa: <http://www.w3.org/ns/sosa/>.
{?x a :LoudnessObservation} => {?x a :ComfortObservation}.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :LoudnessSensor.} => {?x a :LoudnessObservation.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Loudness.} => {?s a :LoudnessSensor.}.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :TempSensor.} => {?x a :TempObservation.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Temperature.} => {?s a :TempSensor.}.
{?x a :TempObservation} => {?x a :ComfortObservation}.
";



//         //assert_eq!(6,store.rules_index.rules.len());
//
//         let query = "PREFIX : <http://cascading.reasoning.io/edge/>
// PREFIX sosa: <http://www.w3.org/ns/sosa/>
//         Select * WHERE{ ?obs a :ComfortObservation;
//             sosa:hasSimpleResult ?value.
//     }";
//         let query = Query::parse(query, None).unwrap();

        // define event shape
        let event = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";

        let mut store = TripleStore::new();
        static_triples.into_iter().for_each(|t|store.add(t));
        store.load_rules(rules);

        //load the event shape
        store.load_triples(event,Syntax::NQuads);

        let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://cascading.reasoning.io/edge/LoudnessObservation>".to_string());

        let mut new_rules = Vec::new();
        let  bindings = rewrite_rules(&store.triple_index, &store.rules_index, &backward_head,  None, &mut new_rules);
        let expected = "{?x <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_2_office0>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/LoudnessObservation>.\n}.\n";
        assert_eq!(expected, TripleStore::decode_rules(&new_rules));


    }

    #[test]
    fn test_rewrite_with_reasoning_single_hierarchy(){
        // generate static data
        let num_offices: usize = 1;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

        let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

        let rules ="@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
@prefix : <http://cascading.reasoning.io/edge/>.
@prefix sosa: <http://www.w3.org/ns/sosa/>.
{ ?x sosa:madeBySensor ?s. ?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Loudness.} => {?x a :MyObservation.}.
{ ?x sosa:madeBySensor ?s. ?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Temperature.} => {?x a :MyObservation.}.
";



        // define event shape
        let event = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";

        let mut store = TripleStore::new();
        static_triples.into_iter().for_each(|t|store.add(t));
        store.load_rules(rules);

        //load the event shape
        store.load_triples(event,Syntax::NQuads);

        let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://cascading.reasoning.io/edge/MyObservation>".to_string());

        let mut new_rules = Vec::new();
        let  bindings = rewrite_rules(&store.triple_index, &store.rules_index, &backward_head,  None, &mut new_rules);
        println!("new_rules {:?}",TripleStore::decode_rules(&new_rules));
        let expected = "{?x <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_2_office0>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/MyObservation>.\n}.\n{?x <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_3_office0>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/MyObservation>.\n}.\n";
        assert_eq!(expected, TripleStore::decode_rules(&new_rules));

    }
    #[test]
    fn test_rewrite_with_reasoning_hierarchy_with_rules(){
        // generate static data
        let num_offices: usize = 1;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

        let static_triples = generate_building(num_offices, properties.to_vec(), Some("<http://static/>".to_string()));

        let rules ="@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
@prefix : <http://cascading.reasoning.io/edge/>.
@prefix sosa: <http://www.w3.org/ns/sosa/>.
{?x a :LoudnessObservation} => {?x a :ComfortObservation}.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :LoudnessSensor.} => {?x a :LoudnessObservation.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Loudness.} => {?s a :LoudnessSensor.}.
{?x a sosa:Observation. ?x sosa:madeBySensor ?s. ?s a :TempSensor.} => {?x a :TempObservation.}.
{?s a sosa:Sensor. ?s sosa:observes ?p. ?p a :Temperature.} => {?s a :TempSensor.}.
{?x a :TempObservation} => {?x a :ComfortObservation}.
";




        // define event shape
        let event = "_:b0 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.some.com/Observation> <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/madeBySensor> _:b1 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/hasSimpleResult> _:b2 <http://stream/>.
_:b0 <http://www.w3.org/ns/sosa/observedProperty> _:b3 <http://stream/>.
";

        let mut store = TripleStore::new();
        static_triples.into_iter().for_each(|t|store.add(t));
        store.load_rules(rules);

        //load the event shape
        store.load_triples(event,Syntax::NQuads);

        let backward_head = Triple::from("?x".to_string(),"<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),"<http://cascading.reasoning.io/edge/ComfortObservation>".to_string());

        let mut new_rules = Vec::new();
        let  bindings = rewrite_rules(&store.triple_index, &store.rules_index, &backward_head,  None, &mut new_rules);
        println!("new_rules {:?}",TripleStore::decode_rules(&new_rules));
        let expected = "{?x <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_2_office0>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/LoudnessObservation>.\n}.\n{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/LoudnessObservation>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/ComfortObservation>.\n}.\n{?x <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_3_office0>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/TempObservation>.\n}.\n{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/TempObservation>.\n}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/ComfortObservation>.\n}.\n";
        assert_eq!(expected, TripleStore::decode_rules(&new_rules));

    }
}