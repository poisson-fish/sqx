extern crate pest;
extern crate pest_derive;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/parsers/grammars/netstat_linux.pest"]
pub struct NetstatLinuxParser;

#[cfg(test)]
mod tests {
 
    use pest::Parser;

    use crate::parsers::netstat_linux::NetstatLinuxParser;
    use crate::parsers::netstat_linux::Rule;

    #[test]
    fn test_parse() {
        let parseStr = 
r#"Active Internet connections (w/o servers)
Proto Recv-Q Send-Q Local Address           Foreign Address         State      
tcp        0    276 DARKSTAR:ssh            192.168.1.188:52840     ESTABLISHED
tcp        0      0 localhost:39974         localhost:45487         ESTABLISHED
tcp        0      0 localhost:45487         localhost:39958         ESTABLISHED
tcp        0      0 localhost:39958         localhost:45487         ESTABLISHED
tcp        0      0 localhost:45487         localhost:44612         TIME_WAIT  
tcp        0      0 localhost:45487         localhost:39974         ESTABLISHED
Active UNIX domain sockets (w/o servers)
Proto RefCnt Flags       Type       State         I-Node   Path
unix  2      [ ]         DGRAM                    49404    /run/user/1000/systemd/notify
unix  2      [ ]         DGRAM                    22539    /var/run/chrony/chronyd.sock
unix  3      [ ]         DGRAM      CONNECTED     17488    /run/systemd/notify
unix  7      [ ]         DGRAM      CONNECTED     17503    /run/systemd/journal/dev-log
unix  6      [ ]         DGRAM      CONNECTED     17505    /run/systemd/journal/socket"#;
        let parser = NetstatLinuxParser::parse(Rule::netstat,parseStr);
        println!("{:#?}", parser);
        assert_eq!(1 + 2, 3);
    }
}