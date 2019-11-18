use caroline::types::AWS::EC2::VPC::VPCBuilder;
use serde_yaml;

#[test]
fn test_vpc_ec2() {
    let vpc = VPCBuilder::default()
        .EnableDnsSupport(true)
        .build()
        .unwrap();

    let v = serde_yaml::to_string(&vpc).unwrap();
    println!("{}", v);
}
