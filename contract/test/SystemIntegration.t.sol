// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";

import "../src/TokenTreasuryUpgradeable.sol";
import "../src/CAssetDTokenWrapUpgradeable.sol";
import "../src/CAssetDTokenWrapFactoryUpgradeable.sol";
import "../src/CAssetDTokenWrapRouterUpgradeable.sol";
import "../mocks/MockERC20.sol";

contract SystemIntegrationTest is Test {
    MockERC20 dToken;
    MockERC20 cAsset;

    TokenTreasuryUpgradeable treasuryImpl;
    CAssetDTokenWrapUpgradeable cWrapImpl;
    CAssetDTokenWrapFactoryUpgradeable factory;
    CAssetDTokenWrapRouterUpgradeable router;

    address owner = address(0xAAA1);
    address user = address(0xBEEF);

    CAssetDTokenWrapUpgradeable deployedWrap;
    address wrapAddr;

    function setUp() public {
        // deploy mocks
        dToken = new MockERC20("TokenA", "A");
        cAsset = new MockERC20("TokenB", "B");

        // deploy implementations
        treasuryImpl = new TokenTreasuryUpgradeable();
        treasuryImpl.initialize(address(0),address(0),address(0)); // dummy init, clones will init correctly

        cWrapImpl = new CAssetDTokenWrapUpgradeable();

        router = new CAssetDTokenWrapRouterUpgradeable();
        router.initialize(owner);

        factory = new CAssetDTokenWrapFactoryUpgradeable();
        factory.initialize(address(treasuryImpl), address(cWrapImpl), address(router), owner);

        // deploy one wrap pair from factory
        vm.prank(owner);
        wrapAddr = factory.deployWrap(address(dToken), address(cAsset),18, 18, 30, 30);
        deployedWrap = CAssetDTokenWrapUpgradeable(wrapAddr);

        // register pair in router
        vm.prank(owner);
        router.registerWrapper(wrapAddr, address(dToken), address(cAsset));

        address dTokenTreasury = deployedWrap.dTokenTreasury();

        dToken.mint(address(this), 1_000 ether); // treasury seed simulation
        // initial deposit dTokens
        dToken.approve(address(dTokenTreasury), 1_000 ether);
        TokenTreasuryUpgradeable(dTokenTreasury).deposit(1_000 ether);
    }

    function testWrapMovesFundsToTreasury() public {
        uint256 amount = 10 ether;
        deal(address(cAsset), user, 1_000 ether, true);

        vm.startPrank(user);
        // we must first unwrap the cAsset cause by definition initially the sc has only dTokens deposit
        cAsset.approve(address(router), amount);
        router.unwrap(address(cAsset), amount, address(dToken));

        address dTokenTreasury = deployedWrap.dTokenTreasury();
        address cAssetTreasury = deployedWrap.cAssetTreasury();

        assertEq(dToken.balanceOf(dTokenTreasury), 1_000 ether - amount);
        assertEq(cAsset.balanceOf(cAssetTreasury), amount);
        assertEq(cAsset.balanceOf(user), 1_000 ether - amount);
        vm.stopPrank();
    }

    function testUnwrapReturnsDToken() public {
        deal(address(cAsset), user, 1_000 ether, true);
        vm.startPrank(user);

        uint256 amount = 10 ether;
        // we can directly unwrap the cAsset cause by definition initially the sc has only dTokens deposit
        cAsset.approve(address(router), amount);
        router.unwrap(address(cAsset), amount, address(dToken));

        assertEq(dToken.balanceOf(user), (amount*9970)/10_000);  // got dToken back subtracted by fee
        assertEq(cAsset.balanceOf(user), 1_000 ether - amount);
        vm.stopPrank();
    }

    function testFactoryRegistersWrap() public view{
        address[] memory all = factory.getAllWraps();
        assertEq(all.length, 1);
        assertEq(all[0], wrapAddr);
    }
}
