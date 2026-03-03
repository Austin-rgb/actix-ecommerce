use async_trait::async_trait;
use inventory::InventoryModule;
use messaging::MessagingModule;
use orders::{InventoryAgent, OrderEvent, OrderEventHandler};

pub struct OrdersInventoryAgent {
    pub inventory_module: InventoryModule,
}

#[async_trait]
impl InventoryAgent for OrdersInventoryAgent {
    async fn reserve(&self, item: String, qty: u32) -> bool {
        match self.inventory_module.service.reserve_item(&item, qty).await {
            Ok(r) => true,
            Err(e) => false,
        }
    }
}

pub struct EventMessanger {
    pub messenger: MessagingModule,
}
#[async_trait]
impl OrderEventHandler for EventMessanger {
    async fn handle(&self, event: OrderEvent) {
        let message = match event {
            OrderEvent::Created(order) => (
                "order received successfully: order-id = ".to_owned() + &order.0,
                order.1,
            ),
            OrderEvent::Confirmed(order) => (
                "order confirmed successfully: order-id = ".to_owned() + &order.0,
                order.1,
            ),
            OrderEvent::Cancelled(order) => (
                "order cancelled successfully: order-id = ".to_owned() + &order.0,
                order.1,
            ),
            OrderEvent::Delivered(order) => (
                "order delivered successfully: order-id = ".to_owned() + &order.0,
                order.1,
            ),
        };
        let _ = self
            .messenger
            .state
            .peer_message(
                "admin",
                message.1,
                messaging::CreateMessage {
                    text: message.0,
                    reply_to: None,
                },
            )
            .await;
    }
}
