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

fn get_wireguard_output_of_endpoint() -> Option<String>{
    let out = Command::new("wg show").output().expect("Failed to execute command: wg show");
    let str_out = String::from_utf8(out.stdout).expect("error changed utf8");
    if let Some(idx) = str_out.find("endpoint"){
        let after_tag = &str_out[idx..];
        if let Some(ip_begin) = after_tag.find(char::is_numeric){
            if let Some(ip_end) = after_tag.find(":"){
                return Some(str_out[idx + ip_begin .. idx + ip_end].to_string());
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
                println!("no addresses returned!");
            }
        }
        None
    }
}

fn run(cfg_name: &str, domain: &str) -> !{
    const SECONDS: u64 = 5;
    let resolver = MyResolver::new();
    let restart_cmd = format!("wg-quick down {0}; wg-quick up {0}", cfg_name);
    loop{
        if let Some(ip) = get_wireguard_output_of_endpoint(){
            if let Some(ip_now) = resolver.gethostbyname(domain){
                if ip != ip_now{
                    println!("wireguard endpoint old ip is {}, new ip is {}, so restart {}", ip, ip_now, cfg_name);
                    Command::new(&restart_cmd).output().ok();
                }
            }
        }
        sleep(Duration::from_secs(SECONDS));
    }
}

fn parse_args() -> String{
    let matches = App::new("wireguard ddns")
                          .version("0.1")
                          .author("persy")
                          .about("auto restart wireguard for ddns")
                          .arg(Arg::with_name("wg_cfg")
                               .short("c")
                               .value_name("FILE")
                               .help("set wireguard config path")
                               .takes_value(true))
                          .get_matches();
    let cfg = matches.value_of("wg_cfg").unwrap_or("/etc/wireguard/wg0.conf");
    cfg.to_string()
}

fn get_domain_from_wg_conf(cfg_path: &String) -> String{
    let contents = fs::read_to_string(cfg_path).unwrap();
    if let Some(idx) = contents.find("Endpoint"){
        let contents = &contents[idx..];
        if let Some(idx) = contents.find("="){
            let contents = &contents[idx + 1..];
            if let Some(end) = contents.find(":"){
                return contents[..end].trim().to_string();
            }
        }
    }
    panic!("can't find endpoint domain from wireguard config:{}", cfg_path);
}

fn main() {
    let cfg_path = parse_args();
    let domain = get_domain_from_wg_conf(&cfg_path);
    let path = Path::new(&cfg_path[..]);
    let file_name = path.file_stem().unwrap();
    run(file_name.to_str().unwrap(), &domain[..]);
}
