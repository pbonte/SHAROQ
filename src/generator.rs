use roxi::parser::{Parser, Syntax};
use roxi::triples::Triple;

static URI: &str = "http://cascading.reasoning.io/edge/";

pub fn generate_observation(observation_id: &str, sensor_id: &str, value: &str, stream_name: Option<String>) -> String{
    let observation_uri= format!("<{URI}observation_{observation_id}>");
    let stream_name = stream_name.clone().unwrap_or_else(|| "".to_string());
    let type_def = format!("{observation_uri} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Observation> {stream_name}.");
    let sensor_link = format!("{observation_uri} <http://www.w3.org/ns/sosa/madeBySensor> <{URI}sensor_{sensor_id}> {stream_name}.");
    let value_link = format!("{observation_uri} <http://www.w3.org/ns/sosa/hasSimpleResult> \"{value}\" {stream_name}.");

    format!("{type_def}\n{sensor_link}\n{value_link}")
}

// <http://cascading.reasoning.io/edge/prop_CO2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/CO2> <http://static/>.
pub fn generate_office(office_id: &str, sensors: Vec<&str>, properties: Vec<&str>, connected_to: Vec<&str>,graph_name: Option<String>) -> String {
    let graph_name = graph_name.clone().unwrap_or_else(|| "".to_string());
    let office_uri= format!("<{URI}office_{office_id}>");
    let mut office_def = format!("{office_uri} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{URI}Office> {graph_name}.");
    office_def.push_str(format!("\n<{URI}office_{office_id}> <{URI}hasName> \"{office_id}\" {graph_name}.").as_str());
    for connected_office in connected_to{
        office_def.push_str(format!("\n<{URI}office_{office_id}> <{URI}connectedTo> <{URI}office_{connected_office}> {graph_name}.").as_str());
    }
    for (sensor_id, property_id) in sensors.iter().zip(properties.iter()){
        let sensor_uri= format!("<{URI}sensor_{sensor_id}>");
        office_def.push_str(format!("\n{sensor_uri} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Sensor> {graph_name}.").as_str());
        office_def.push_str(format!("\n{sensor_uri} <http://www.w3.org/ns/sosa/observes> <{URI}prop_{property_id}> {graph_name}.").as_str());
        office_def.push_str(format!("\n{sensor_uri} <{URI}hasLocation> <{URI}office_{office_id}> {graph_name}.").as_str());
        office_def.push_str(format!("\n<{URI}prop_{property_id}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{URI}{property_id}> {graph_name}.").as_str());

    }


    office_def
}
pub fn generate_building(num_offices: usize, properties: Vec<&str>, graph_name: Option<String>) -> Vec<Triple> {
    let mut triples = Vec::new();
    for office_id in 0..num_offices {
        let sensors: Vec<String> = (0..properties.len()).map(|x| format!("{x}_office{office_id}")).collect();
        let prev_office = if office_id > 0 { office_id - 1 } else { num_offices };
        let next_office = office_id + 1;
        let connected_to = [format!("office{prev_office}"), format!("office{next_office}")];
        let office_triples = generate_office(format!("office{office_id}").as_str(), sensors.iter().map(|x| x.as_str()).collect(), properties.to_vec(), connected_to.iter().map(|x| x.as_str()).collect(), graph_name.clone());
        triples.append(&mut Parser::parse_triples(&office_triples, Syntax::NQuads).unwrap());
    }
    triples
}
#[cfg(test)]
mod test{
    use std::collections::HashSet;
    use roxi::parser::{Parser, Syntax};
    use roxi::sparql::{Binding, eval_query, evaluate_plan_and_debug};
    use roxi::tripleindex::TripleIndex;
    use roxi::triples::Triple;
    use crate::generator::{generate_building, generate_observation, generate_office};
    use spargebra::Query;

    #[test]
    fn test_generate_observation() {
        let observation = generate_observation("obs1", "sensor1", "100", None);
        let expected =  "<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Observation> .
<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_sensor1> .
<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/ns/sosa/hasSimpleResult> \"100\" .";
        assert_eq!(expected.to_string(), observation);

        let observation = generate_observation("obs1", "sensor1", "100", Some("<http://stream.name>".to_string()));
        let expected =  "<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Observation> <http://stream.name>.
<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/ns/sosa/madeBySensor> <http://cascading.reasoning.io/edge/sensor_sensor1> <http://stream.name>.
<http://cascading.reasoning.io/edge/observation_obs1> <http://www.w3.org/ns/sosa/hasSimpleResult> \"100\" <http://stream.name>.";
        assert_eq!(expected.to_string(), observation);

        let triples = Parser::parse_triples(&observation, Syntax::NQuads).unwrap();
        assert_eq!(3, triples.len());
    }

    #[test]
    fn test_generate_office(){
        let office = generate_office("office1", ["sensor1"].to_vec(), ["CO2"].to_vec(),["office2"].to_vec(),None);
        let expected = "<http://cascading.reasoning.io/edge/office_office1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/Office> .
<http://cascading.reasoning.io/edge/office_office1> <http://cascading.reasoning.io/edge/hasName> \"office1\" .
<http://cascading.reasoning.io/edge/office_office1> <http://cascading.reasoning.io/edge/connectedTo> <http://cascading.reasoning.io/edge/office_office2> .
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Sensor> .
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://www.w3.org/ns/sosa/observes> <http://cascading.reasoning.io/edge/prop_CO2> .
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://cascading.reasoning.io/edge/hasLocation> <http://cascading.reasoning.io/edge/office_office1> .
<http://cascading.reasoning.io/edge/prop_CO2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/CO2> .";
        assert_eq!(expected.to_string(), office);

        let office = generate_office("office1", ["sensor1", "sensor2"].to_vec(), ["CO2", "Humidity"].to_vec(),["office2", "office3"].to_vec(),Some("<http://static/>".to_string()));
        let expected = "<http://cascading.reasoning.io/edge/office_office1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/Office> <http://static/>.
<http://cascading.reasoning.io/edge/office_office1> <http://cascading.reasoning.io/edge/hasName> \"office1\" <http://static/>.
<http://cascading.reasoning.io/edge/office_office1> <http://cascading.reasoning.io/edge/connectedTo> <http://cascading.reasoning.io/edge/office_office2> <http://static/>.
<http://cascading.reasoning.io/edge/office_office1> <http://cascading.reasoning.io/edge/connectedTo> <http://cascading.reasoning.io/edge/office_office3> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Sensor> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://www.w3.org/ns/sosa/observes> <http://cascading.reasoning.io/edge/prop_CO2> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor1> <http://cascading.reasoning.io/edge/hasLocation> <http://cascading.reasoning.io/edge/office_office1> <http://static/>.
<http://cascading.reasoning.io/edge/prop_CO2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/CO2> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor2> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/sosa/Sensor> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor2> <http://www.w3.org/ns/sosa/observes> <http://cascading.reasoning.io/edge/prop_Humidity> <http://static/>.
<http://cascading.reasoning.io/edge/sensor_sensor2> <http://cascading.reasoning.io/edge/hasLocation> <http://cascading.reasoning.io/edge/office_office1> <http://static/>.
<http://cascading.reasoning.io/edge/prop_Humidity> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://cascading.reasoning.io/edge/Humidity> <http://static/>.";
        assert_eq!(expected.to_string(), office);

    }
    #[test]
    fn test_generate_building(){
        let num_offices: usize = 2;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];
        let num_office_striples = 2 + 2;
        let num_sensor_triples = 4 * properties.len();
        let expected_triples = num_offices * (num_office_striples + num_sensor_triples);
        let triples = generate_building(num_offices, properties.to_vec(), None);

        assert_eq!(expected_triples, triples.len());
    }

    #[test]
    fn evaluation_increase_static(){
        let num_offices: usize = 2;
        let properties = ["CO2", "Humidity", "Loudness", "Temperature"];

        let static_triples = generate_building(num_offices, properties.to_vec(), None);

        let observations = generate_observation("obs1", "0_office1", "100", None);
        let observation_triples = Parser::parse_triples(&observations, Syntax::NQuads).unwrap();
        let mut index = TripleIndex::new();
        static_triples.into_iter().for_each(|t| index.add(t));
        observation_triples.into_iter().for_each(|t| index.add(t));

        let query_str = "Select * WHERE { ?obs a <http://www.w3.org/ns/sosa/Observation>;
<http://www.w3.org/ns/sosa/hasSimpleResult> ?value;
<http://www.w3.org/ns/sosa/madeBySensor> ?sensor.
?sensor <http://cascading.reasoning.io/edge/hasLocation> ?loc.
?loc <http://cascading.reasoning.io/edge/hasName> \"office1\".
 }";
        let query = Query::parse(query_str, None).unwrap();
        let plan = eval_query(&query, &index);
        let iterator = evaluate_plan_and_debug(&plan, &index);

        assert_eq!(1, iterator.collect::<HashSet<Vec<Binding>>>().len());
    }


}