// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {CAssetDTokenWrapUpgradeable} from "./CAssetDTokenWrapUpgradeable.sol";

contract CAssetDTokenWrapRouterUpgradeable is Initializable, AccessControlUpgradeable, UUPSUpgradeable, ReentrancyGuardUpgradeable {
    using SafeERC20 for IERC20;

    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    mapping(address => bool) public isRegisteredWrap;
    mapping(bytes32 => address) public tokenWrapAddress;

    event WrapperRegistered(
        address indexed wrapper,
        address indexed dToken,
        address indexed cAsset
    );

    event WrapperUnregistered(
        address indexed wrapper,
        address indexed dToken,
        address indexed cAsset
    );

    event Wrapped(
        address indexed caller,
        address indexed wrapper,
        address dToken,
        uint256 amount,
        address cAsset
    );

    event Unwrapped(
        address indexed caller,
        address indexed wrapper,
        address cAsset,
        uint256 amount,
        address dToken
    );

    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MANAGER_ROLE, admin);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    function registerWrapper(address wrapper, address dToken, address cAsset) external onlyRole(MANAGER_ROLE) {
        require(wrapper != address(0), "Router: zero wrapper");
        require(dToken != address(0) && cAsset != address(0), "Router: zero token");
        bytes32 key = _pairKey(dToken, cAsset);
        require(tokenWrapAddress[key] == address(0), "Router: pair already registered");

        tokenWrapAddress[key] = wrapper;
        isRegisteredWrap[wrapper] = true;

        emit WrapperRegistered(wrapper, dToken, cAsset);
    }

    function unregisterWrapper(address dToken, address cAsset) external onlyRole(MANAGER_ROLE) {
        bytes32 key = _pairKey(dToken, cAsset);
        address wrapper = tokenWrapAddress[key];
        require(wrapper != address(0), "Router: pair not registered");

        delete tokenWrapAddress[key];
        delete isRegisteredWrap[wrapper];

        emit WrapperUnregistered(wrapper, dToken, cAsset);
    }

    function wrap(address dToken, uint256 amount, address cAsset) external nonReentrant {
        require(amount > 0, "Router: zero amount");
        bytes32 key = _pairKey(dToken, cAsset);
        address wrapper = tokenWrapAddress[key];
        require(wrapper != address(0), "Router: Wrapper not registered");

        uint256 balanceBefore = IERC20(dToken).balanceOf(wrapper);
        IERC20(dToken).safeTransferFrom(msg.sender, wrapper, amount);
        uint256 received = IERC20(dToken).balanceOf(wrapper) - balanceBefore;
        require(received > 0, "Router: transfer failed");

        CAssetDTokenWrapUpgradeable(wrapper).wrap(msg.sender, received);
        emit Wrapped(msg.sender, wrapper, dToken, amount, cAsset);
    }

    function unwrap(address cAsset, uint256 amount, address dToken) external nonReentrant {
        require(amount > 0, "Router: zero amount");
        bytes32 key = _pairKey(dToken, cAsset);
        address wrapper = tokenWrapAddress[key];
        require(wrapper != address(0), "Router: Wrapper not registered");

        uint256 balanceBefore = IERC20(cAsset).balanceOf(wrapper);
        IERC20(cAsset).safeTransferFrom(msg.sender, wrapper, amount);
        uint256 received = IERC20(cAsset).balanceOf(wrapper) - balanceBefore;
        require(received > 0, "Router: transfer failed");

        CAssetDTokenWrapUpgradeable(wrapper).unwrap(msg.sender, received);
        emit Unwrapped(msg.sender, wrapper, cAsset, amount, dToken);
    }

    function _pairKey(address dToken, address cAsset) internal pure returns (bytes32 result) {
        assembly {
            mstore(0x00, dToken)
            mstore(0x20, cAsset)
            result := keccak256(0x00, 0x40)
        }
    }
}
