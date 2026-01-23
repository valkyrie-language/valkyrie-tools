use valkyrie_errors::ValkyrieResult;

use valor_config::ValorConfig;

#[test]
fn test() -> ValkyrieResult {
    toml::from_str::<ValorConfig>(include_str!("deps2.toml"))?;
    json5::from_str::<ValorConfig>(include_str!("deps1.json5"))?;
    Ok(())
}

#[test]
fn test2() -> ValkyrieResult {
    println!("{:#?}", ValorConfig::load("tests/mock1")?);
    Ok(())
}
