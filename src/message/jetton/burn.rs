use num_bigint::BigUint;

use super::JETTON_BURN;
use crate::address::TonAddress;
use crate::cell::{ArcCell, Cell, CellBuilder};
use crate::message::{InvalidMessage, TonMessageError};
/// Creates a body for jetton burn according to TL-B schema:
///
/// ```raw
/// burn#595f07bc query_id:uint64 amount:(VarUInteger 16)
///               response_destination:MsgAddress custom_payload:(Maybe ^Cell)
///               = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct JettonBurnMessage {
    /// arbitrary request number.
    pub query_id: u64,
    /// amount of burned jettons
    pub amount: BigUint,
    /// address where to send a response with confirmation of a successful burn and the rest of the incoming message coins.
    pub response_destination: TonAddress,
    /// optional custom data (which is used by either sender or receiver jetton wallet for inner logic).
    pub custom_payload: Option<ArcCell>,
}

impl JettonBurnMessage {
    pub fn new(amount: &BigUint) -> Self {
        JettonBurnMessage {
            query_id: 0,
            amount: amount.clone(),
            response_destination: TonAddress::null(),
            custom_payload: None,
        }
    }

    pub fn with_query_id(&mut self, query_id: u64) -> &mut Self {
        self.query_id = query_id;
        self
    }

    pub fn with_response_destination(&mut self, response_destination: &TonAddress) -> &mut Self {
        self.response_destination = response_destination.clone();
        self
    }

    pub fn with_custom_payload(&mut self, custom_payload: ArcCell) -> &mut Self {
        self.custom_payload = Some(custom_payload);
        self
    }

    pub fn build(&self) -> Result<Cell, TonMessageError> {
        let mut message = CellBuilder::new();
        message.store_u32(32, JETTON_BURN)?;
        message.store_u64(64, self.query_id)?;
        message.store_coins(&self.amount)?;
        message.store_address(&self.response_destination)?;
        message.store_maybe_cell_ref(&self.custom_payload)?;

        Ok(message.build()?)
    }

    pub fn parse(cell: &Cell) -> Result<Self, TonMessageError> {
        let mut parser = cell.parser();

        let opcode: u32 = parser.load_u32(32)?;
        let query_id = parser.load_u64(64)?;
        if opcode != JETTON_BURN {
            let invalid = InvalidMessage {
                opcode: Some(opcode),
                query_id: Some(query_id),
                message: format!("Unexpected opcode.  {0:08x} expected", JETTON_BURN),
            };
            return Err(TonMessageError::InvalidMessage(invalid));
        }
        let amount = parser.load_coins()?;
        let response_destination = parser.load_address()?;
        let custom_payload = parser.load_maybe_cell_ref()?;
        parser.ensure_empty()?;

        let result = JettonBurnMessage {
            query_id,
            amount,
            response_destination,
            custom_payload,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigUint;

    use crate::address::TonAddress;
    use crate::cell::BagOfCells;
    use crate::message::{JettonBurnMessage, TonMessageError};

    const JETTON_BURN_WITH_CUSTOM_PAYLOAD_INDICATOR_MSG: &str =  "b5ee9c72010101010033000062595f07bc0000009b5946deef3080f21800b026e71919f2c839f639f078d9ee6bc9d7592ebde557edf03661141c7c5f2ea2";
    const NOT_BURN: &str = "b5ee9c72010101010035000066595f07bc0000000000000001545d964b800800cd324c114b03f846373734c74b3c3287e1a8c2c732b5ea563a17c6276ef4af30";

    #[test]
    fn test_jetton_burn_parser() -> Result<(), TonMessageError> {
        let boc_with_indicator =
            BagOfCells::parse_hex(JETTON_BURN_WITH_CUSTOM_PAYLOAD_INDICATOR_MSG).unwrap();
        let cell_with_indicator = boc_with_indicator.single_root().unwrap();
        let result_jetton_transfer_msg_with_indicator: JettonBurnMessage =
            JettonBurnMessage::parse(cell_with_indicator)?;

        let expected_jetton_transfer_msg = JettonBurnMessage {
            query_id: 667217747695,
            amount: BigUint::from(528161u64),
            response_destination: TonAddress::from_str(
                "EQBYE3OMjPlkHPsc-Dxs9zXk66yXXvKr9vgbMIoOPi-XUa-f",
            )
            .unwrap(),
            custom_payload: None,
        };

        assert_eq!(
            expected_jetton_transfer_msg,
            result_jetton_transfer_msg_with_indicator
        );

        let boc = BagOfCells::parse_hex(NOT_BURN).unwrap();
        let cell = boc.single_root().unwrap();

        let result_jetton_transfer_msg = JettonBurnMessage::parse(cell)?;

        let expected_jetton_transfer_msg = JettonBurnMessage {
            query_id: 1,
            amount: BigUint::from(300000000000u64),
            response_destination: TonAddress::from_str(
                "EQBmmSYIpYH8IxubmmOlnhlD8NRhY5la9SsdC-MTt3pXmOSI",
            )
            .unwrap(),
            custom_payload: None,
        };

        assert_eq!(expected_jetton_transfer_msg, result_jetton_transfer_msg);
        Ok(())
    }

    #[test]
    fn test_jetton_burn_builder() {
        let result_cell = JettonBurnMessage::new(&BigUint::from(528161u64))
            .with_query_id(667217747695)
            .with_response_destination(
                &TonAddress::from_str("EQBYE3OMjPlkHPsc-Dxs9zXk66yXXvKr9vgbMIoOPi-XUa-f").unwrap(),
            )
            .build()
            .unwrap();

        let result_boc_serialized = BagOfCells::from_root(result_cell).serialize(false).unwrap();
        let expected_boc_serialized =
            hex::decode(JETTON_BURN_WITH_CUSTOM_PAYLOAD_INDICATOR_MSG).unwrap();

        assert_eq!(expected_boc_serialized, result_boc_serialized);

        let result_cell = JettonBurnMessage {
            query_id: 1,
            amount: BigUint::from(300000000000u64),
            response_destination: TonAddress::from_str(
                "EQBmmSYIpYH8IxubmmOlnhlD8NRhY5la9SsdC-MTt3pXmOSI",
            )
            .unwrap(),
            custom_payload: None,
        }
        .build()
        .unwrap();

        let result_boc_serialized = BagOfCells::from_root(result_cell).serialize(false).unwrap();
        let expected_boc_serialized = hex::decode(NOT_BURN).unwrap();

        assert_eq!(expected_boc_serialized, result_boc_serialized);
    }
}
