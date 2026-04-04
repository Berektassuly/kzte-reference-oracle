param(
    [string]$ConfigPath = "config/feeder.example.toml"
)

cargo run -p kzte-feeder -- --config $ConfigPath --once
