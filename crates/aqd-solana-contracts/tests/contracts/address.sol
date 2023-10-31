contract AddressTest {
    address public addressData;

    @payer(payer)
    constructor(address initAddress) {
        addressData = initAddress;
    }

    function getAddressData() public view returns (address) {
        return addressData;
    }
}
