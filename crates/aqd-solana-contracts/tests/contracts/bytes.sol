contract byte_test {
    bytes public byteVal;

    @payer(payer)
    @space(1024)
    constructor(bytes initbyteVal) {
        byteVal = initbyteVal;
    }

    function getbyteVal() public view returns (bytes) {
        return byteVal;
    }
}