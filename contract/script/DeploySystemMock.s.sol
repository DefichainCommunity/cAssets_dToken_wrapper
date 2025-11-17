// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import "../src/TokenTreasuryUpgradeable.sol";
import "../src/CAssetDTokenWrapUpgradeable.sol";
import "../src/CAssetDTokenWrapFactoryUpgradeable.sol";
import "../src/CAssetDTokenWrapRouterUpgradeable.sol";
import "../mocks/MockERC20.sol";

contract DeploySystem is Script {
    function run() external {
        vm.startBroadcast();

        // Step 1: Deploy mock tokens
        MockERC20 tokenA = new MockERC20("dUSDC", "dUSDC");
        MockERC20 tokenB = new MockERC20("cUSDC", "cUSDC");

        console.log("tokenA:", address(tokenA));
        console.log("tokenB:", address(tokenB));

        // Step 2: Deploy logic implementations
        TokenTreasuryUpgradeable tokenTreasuryImpl = new TokenTreasuryUpgradeable();
        CAssetDTokenWrapUpgradeable wrapImpl = new CAssetDTokenWrapUpgradeable();
        console.log("TokenTreasury impl:", address(tokenTreasuryImpl));
        console.log("CAssetDTokenWrap impl:", address(wrapImpl));

        // Step 3: Deploy router
        CAssetDTokenWrapRouterUpgradeable router = new CAssetDTokenWrapRouterUpgradeable();
        router.initialize(msg.sender);
        console.log("Router:", address(router));

        // Step 4: Deploy factory (UUPS)
        CAssetDTokenWrapFactoryUpgradeable factory = new CAssetDTokenWrapFactoryUpgradeable();
        factory.initialize(address(tokenTreasuryImpl), address(wrapImpl), address(router), msg.sender);
        console.log("Factory:", address(factory));

        // Step 5: Deploy a new wrap via factory
        address wrapAddr = factory.deployWrap(address(tokenA), address(tokenB),18, 18, 30, 40);
        console.log("Wrap instance:", wrapAddr);

        // Step 6: Register with router
        router.registerWrapper(wrapAddr, address(tokenA), address(tokenB));

        // Optional: fund and approve tokens
        tokenA.mint(msg.sender, 1000 ether);
        tokenB.mint(address(0x427cf764eb44f523F12798Ae48388F7f1c33277b), 1000 ether);

        address dTokenTreasury = CAssetDTokenWrapUpgradeable(wrapAddr).dTokenTreasury();
        /* tokenA.approve(address(router), type(uint256).max); */
        tokenA.approve(address(dTokenTreasury), 1_000 ether);
        TokenTreasuryUpgradeable(dTokenTreasury).deposit(1_000 ether);

        /* tokenB.approve(address(router), type(uint256).max); */

        console.log("Setup complete!");

        vm.stopBroadcast();
    }
}
