// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

import {TokenTreasuryUpgradeable} from "./TokenTreasuryUpgradeable.sol";

contract CAssetDTokenWrapUpgradeable is Initializable, UUPSUpgradeable, AccessControlUpgradeable, ReentrancyGuardUpgradeable {
    using SafeERC20 for IERC20;

    struct WrapInfo {
        address dTokenAddress;
        address cAssetAddress;
        uint8 dTokenDecimals;
        uint8 cAssetDecimals;
        uint256 dTokenInFeeBps;
        uint256 dTokenOutFeeBps;
    }

    address public dTokenTreasury;
    address public cAssetTreasury;

    address public feeRecipient;
    WrapInfo public info;

    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    bytes32 public constant WRAPPER_ROLE = keccak256("WRAPPER_ROLE");

    event TreasuryCloned(
        address indexed clone,
        address indexed token,
        uint8 indexed slot
    );

    event WrappedEvent(
        address indexed user,
        address dToken,
        address cAsset,
        uint256 amountIn,
        uint256 amountOut
    );

    event UnwrappedEvent(
        address indexed user,
        address cAsset,
        address dToken,
        uint256 amountIn,
        uint256 amountOut
    );

    function initialize(
        address admin,
        address router,
        address treasuryContract,
        WrapInfo calldata _info
    ) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MANAGER_ROLE, admin);
        _grantRole(WRAPPER_ROLE, router);

        feeRecipient = admin;
        info = _info;
        require(info.dTokenInFeeBps <= 100, "dTokenInFee too high (max 1% == 100)");
        require(info.dTokenOutFeeBps <= 100, "dTokenOutFee too high (max 1% == 100)");

        dTokenTreasury = Clones.clone(treasuryContract);
        TokenTreasuryUpgradeable(dTokenTreasury).initialize(address(this), admin, info.dTokenAddress);
        emit TreasuryCloned(dTokenTreasury, info.dTokenAddress, 1);

        cAssetTreasury = Clones.clone(treasuryContract);
        TokenTreasuryUpgradeable(cAssetTreasury).initialize(address(this), admin, info.cAssetAddress);
        emit TreasuryCloned(cAssetTreasury, info.cAssetAddress, 2);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}

    function wrap(address user, uint256 amount) external nonReentrant onlyRole(WRAPPER_ROLE) {
        require(amount > 0, "Wrapper: zero amount");
        require(user != address(0), "Wrapper: zero user");
        require(IERC20(info.dTokenAddress).balanceOf(address(this)) >= amount, "Wrapper: insufficient tokens");

        uint256 fee = (amount * info.dTokenInFeeBps) / 10_000;
        uint256 net = amount - fee;
        uint256 out = _convert(net, info.dTokenDecimals, info.cAssetDecimals);

        IERC20(info.dTokenAddress).safeTransfer(dTokenTreasury, net);
        if (fee > 0) IERC20(info.dTokenAddress).safeTransfer(feeRecipient, fee);

        TokenTreasuryUpgradeable(dTokenTreasury).reclaimNotify(net);
        TokenTreasuryUpgradeable(cAssetTreasury).dispense(user, out);

        emit WrappedEvent(user, info.dTokenAddress, info.cAssetAddress, amount, out);
    }

    function unwrap(address user, uint256 amount) external nonReentrant onlyRole(WRAPPER_ROLE) {
        require(amount > 0, "Wrapper: zero amount");
        require(user != address(0), "Wrapper: zero user");
        require(IERC20(info.cAssetAddress).balanceOf(address(this)) >= amount, "Wrapper: insufficient tokens");

        IERC20(info.cAssetAddress).safeTransfer(cAssetTreasury, amount);
        TokenTreasuryUpgradeable(cAssetTreasury).reclaimNotify(amount);

        uint256 dTokenAmount = _convert(amount, info.cAssetDecimals, info.dTokenDecimals);
        uint256 fee = (dTokenAmount * info.dTokenOutFeeBps) / 10_000;
        uint256 net = dTokenAmount - fee;

        TokenTreasuryUpgradeable(dTokenTreasury).dispense(user, net);
        if (fee > 0) TokenTreasuryUpgradeable(dTokenTreasury).dispense(feeRecipient, fee);

        emit UnwrappedEvent(user, info.cAssetAddress, info.dTokenAddress, amount, net);
    }

    function updateFees(uint256 _dTokenInFeeBps, uint256 _dTokenOutFeeBps) external onlyRole(MANAGER_ROLE) {
        require(_dTokenInFeeBps <= 100, "dTokenInFee too high (max 1% == 100)");
        require(_dTokenOutFeeBps <= 100, "dTokenOutFee too high (max 1% == 100)");
        info.dTokenInFeeBps = _dTokenInFeeBps;
        info.dTokenOutFeeBps = _dTokenOutFeeBps;
    }

    function updateFeeRecipient(address _feeRecipient) external onlyRole(MANAGER_ROLE) {
        feeRecipient = _feeRecipient;
    }

    function _convert(uint256 amount, uint8 decIn, uint8 decOut) internal pure returns (uint256) {
        if (decIn == decOut) return amount;
        else if (decIn > decOut) return amount / (10 ** (decIn - decOut));
        else return amount * (10 ** (decOut - decIn));
    }
}
