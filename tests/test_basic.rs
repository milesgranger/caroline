use caroline::types::AWS::EC2::VPC::VPCBuilder;

#[test]
fn test_vpc_ec2() {
    let vpc = VPCBuilder::default().EnableDnsSupport(true);
}
