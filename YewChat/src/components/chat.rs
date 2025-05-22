use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://api.dicebear.com/9.x/initials/svg?seed={}", // <--- This is the updated URL
                                    u
                                )
                                    .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen">
                // Sidebar for users
                <div class="flex-none w-56 h-screen bg-gray-100 p-3">
                    <div class="text-xl font-bold text-gray-700 mb-4 flex items-center">
                        <img src="https://api.iconify.design/heroicons:users-solid.svg?color=%234a5568" class="w-6 h-6 mr-2" alt="Users Icon"/>
                        {"Online Users"}
                    </div>
                    <p class="text-sm text-gray-500 mb-4">{"Who's here to chat?"}</p>
                    <div class="overflow-y-auto h-5/6"> // Added scroll for user list
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex m-3 bg-white rounded-lg p-2 shadow-sm items-center">
                                        <div>
                                            <img class="w-12 h-12 rounded-full border-2 border-blue-300" src={u.avatar.clone()} alt="avatar"/>
                                        </div>
                                        <div class="flex-grow p-3">
                                            <div class="flex text-sm justify-between font-semibold">
                                                <div>{u.name.clone()}</div>
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                {"Ready to connect!"} // More engaging status
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                // Main chat area
                <div class="grow h-screen flex flex-col bg-gray-50">
                    <div class="w-full h-14 border-b-2 border-gray-200 flex items-center px-4 bg-white shadow-sm">
                        <div class="text-xl font-bold text-gray-700 flex items-center">
                            <img src="https://api.iconify.design/heroicons:chat-bubble-left-right-solid.svg?color=%234a5568" class="w-7 h-7 mr-2" alt="Chat Icon"/>
                            {"Let's Chat!"}
                        </div>
                    </div>
                    <div class="w-full grow overflow-auto p-4 custom-scrollbar"> // Added custom-scrollbar for styling
                        {
                            self.messages.iter().map(|m| {
                                // Create the unknown_user_profile here so it lives long enough
                                let unknown_user_profile = UserProfile {
                                    name: "Unknown".to_string(),
                                    avatar: "https://avatars.dicebear.com/api/adventurer-neutral/unknown.svg".to_string(),
                                };
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap_or(&unknown_user_profile);
                                html!{
                                    <div class={format!("flex items-end {} bg-white rounded-lg p-3 my-4 shadow-sm", if m.from == user.name {"ml-auto rounded-br-none"} else {"mr-auto rounded-tl-none"})}>
                                        <img class="w-9 h-9 rounded-full mr-3 border-2 border-gray-300" src={user.avatar.clone()} alt="avatar"/>
                                        <div>
                                            <div class="text-sm font-semibold text-gray-800 mb-1">
                                                {user.name.as_str()}
                                            </div>
                                            <div class="text-sm text-gray-600 break-words"> // break-words for long messages
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded-md max-w-xs" src={m.message.clone()} alt="gif"/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-16 flex px-4 items-center bg-white border-t-2 border-gray-200 shadow-lg">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type your message here..." class="block w-full py-2 pl-5 pr-3 mx-3 bg-gray-100 rounded-full outline-none focus:ring-2 focus:ring-blue-400 focus:bg-white text-gray-800 transition duration-200" name="message" required=true />
                        <button onclick={submit} class="p-3 shadow-lg bg-blue-600 w-12 h-12 rounded-full flex justify-center items-center text-white hover:bg-blue-700 transition duration-200 transform hover:scale-105">
                            <svg fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-6 h-6">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}