/// @author persy
/// @date 2021/11/10 8:57

use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::fs;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use clap::{Arg, App};
use chrono::offset::Local;

fn get_wireguard_output_of_endpoint() -> Option<String>{
    let out = Command::new("wg").output().expect("Failed to execute command: wg");
    let str_out = String::from_utf8(out.stdout).expect("error changed utf8");
    if let Some(idx) = str_out.find("endpoint"){
        let it = &str_out[idx..];
        if let Some(idx) = it.find(char::is_numeric){
            let it = &it[idx..];
            if let Some(idx) = it.find(":"){
                return Some(it[..idx].trim().to_string());
            }
        }
    }
    None
}

struct MyResolver{
    resolver: Resolver,
}

impl MyResolver{
    fn new() -> Self {
        return Self{ resolver: Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap() };
    }

    fn gethostbyname(&self, domain: &str) -> Option<String>{
        if let Ok(response) = self.resolver.lookup_ip(domain){
            if let Some(address) = response.iter().next(){
                return Some(address.to_string());
            }
            else{
                println!("{}. no addresses returned!", Local::now());
            }
        }
        None
    }
}

fn run(cfg_name: &str, domain: &str) -> !{
    const SECONDS: u64 = 60 * 2;
    let resolver = MyResolver::new();
    loop{
        if let Some(ip) = get_wireguard_output_of_endpoint(){
            if let Some(ip_now) = resolver.gethostbyname(domain){
                if ip != ip_now{
                    println!("{}, wireguard endpoint old ip is {}, new ip is {}, so restart {}", Local::now(), ip, ip_now, cfg_name);
                    Command::new("wg-quick").arg("down").arg(cfg_name).status().ok();
                    Command::new("wg-quick").arg("up").arg(cfg_name).status().ok();
                }
            }
        }
        sleep(Duration::from_secs(SECONDS));
    }
}

fn parse_args() -> Option<String>{
    let matches = App::new("wireguard ddns")
                          .version("0.1.0")
                          .author("persy")
                          .about("auto restart wireguard for ddns")
                          .arg(Arg::with_name("wg_cfg")
                               .short("c")
                               .long("cfg")
                               .value_name("FILE")
                               .help("set wireguard config path")
                               .takes_value(true))
                          .arg(Arg::with_name("service")
                               .short("s")
                               .long("service")
                               .help("set enable service when restart system")
                               .takes_value(false))
                          .get_matches();
    let cfg = matches.value_of("wg_cfg").unwrap_or("/etc/wireguard/wg0.conf");
    if matches.occurrences_of("service") > 0{
        gen_service_cfg("/lib/systemd/system/wg_ddns.service", &cfg.to_string());
        return None;
    }
    Some(cfg.to_string())
}

fn get_domain_from_wg_conf(cfg_path: &String) -> String{
    let contents = fs::read_to_string(cfg_path).expect(&format!("can not open the config file:{}", &cfg_path));
    if let Some(idx) = contents.find("Endpoint"){
        let contents = &contents[idx..];
        if let Some(idx) = contents.find("="){
            let contents = &contents[idx + 1..];
            if let Some(idx) = contents.find(":"){
                return contents[..idx].trim().to_string();
            }
        }
    }
    panic!("can't find endpoint domain from wireguard config:{}", cfg_path);
}

fn gen_service_cfg(service_path: &str, cfg_path: &String){
    let s = format!(r"[Unit]
Description=gen wg_ddns service
After=network.target

[Service]
User=root
ExecStart=wg_ddns -c {}

[Install]
WantedBy=multi-user.target", cfg_path);
    fs::write(service_path, s).unwrap();
    Command::new("systemctl").arg("daemon-reload").status().ok();
    let path = Path::new(&service_path);
    Command::new("systemctl").arg("enable").arg(path.file_name().unwrap().to_str().unwrap()).status().ok();
}

fn main() {
    if let Some(cfg_path) = parse_args(){
        let domain = get_domain_from_wg_conf(&cfg_path);
        let path = Path::new(&cfg_path[..]);
        let file_name = path.file_stem().unwrap();
        run(file_name.to_str().unwrap(), &domain[..]);
    }
}
