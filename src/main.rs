use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PokemonDto {
    id: i32,
    name: String,
    base_experience: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = reqwest::Client::new();
    let pokemon_names = ["pikachu", "arceus", "charmander", "pipi"];
    let mut handles = vec![];

    for pokemon_name in pokemon_names {
        let http_client = http_client.clone();
        let handle = tokio::spawn(async move {
            let url = format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon_name);
            http_client
                .get(&url)
                .send()
                .await?
                .json::<PokemonDto>()
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await? {
            Ok(pokemon_dto) => {
                println!("{pokemon_dto:#?}")
            }
            Err(e) => {
                eprintln!("Error: {}", e)
            }
        }
    }

    Ok(())
}
