use std::{collections::HashMap, str::FromStr, string};
use leptos::{ev::SubmitEvent, html::{Input, Select}, *};
use leptos_meta::*;
use leptos_router::*;
use log::{info, debug};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use wasm_bindgen::JsCast;
use leptos_use::{signal_debounced};
use web_sys::js_sys::Math::sign;

const STORAGE_KEY_LITEM : &str = "litems-key";
const STORAGE_KEY_PARTICIPANTS : &str = "participants-key";


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LItems(pub Vec<LItem>);



impl LItems {
    pub fn new() -> Self {
//        let starting_litems = if cfg!(target_arch = "wasm32") {
//            window()
//            .local_storage()
//            .ok()
//            .flatten()
//            .and_then(|storage| {
//                storage.get_item(STORAGE_KEY_LITEM).ok().flatten().and_then(
//                    |value| serde_json::from_str::<Vec<LItem>>(&value).ok()
//                )
//            })
//        .unwrap_or_default()
//        } else {
//            Vec::new()
//        };
//
        Self(Vec::new())
    }


    pub fn add(&mut self, litem: LItem) {
        self.0.push(litem);
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LItem {
    id: Uuid,
    item_name: RwSignal<String>,
    price: RwSignal<Decimal>,
    participants: RwSignal<Vec<Participant>>,
}


impl LItem {
    fn new(item_name: String, price: Decimal) -> Self {

        let item_name = create_rw_signal(item_name);
        let price = create_rw_signal(price);
        let participants = create_rw_signal(Vec::new());

        LItem {
            id: Uuid::new_v4(),
            item_name,
            price,
            participants
        }
    }




    fn get_split_by_participants(&self) -> Decimal {
        self.price.get() / Decimal::from(self.participants.get().len())
    }

    fn add_participant(&mut self, participant: Participant)  {
        self.participants.update(|p| p.push(participant))
    }

    fn remove_participant(&mut self, participant_id: Uuid) {
        self.participants.update(|p| p.retain(|x| x.id != participant_id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participants(pub Vec<Participant>);




impl Participants {
    pub fn new() -> Self {
//        let starting_participatns = if cfg!(target_arch = "wasm32") {
//            window()
//                .local_storage()
//                .ok()
//                .flatten()
//                .and_then(|storage| {
//                    storage.get_item(STORAGE_KEY_PARTICIPANTS).ok().flatten().and_then(
//                        |value| serde_json::from_str::<Vec<Participant>>(&value).ok(),
//                    )
//                })
//                .unwrap_or_default()
//        } else {
//            Vec::new()
//        };
        Self(Vec::new())
    }



    pub fn add(&mut self, participant: Participant) {
        self.0.push(participant);
    }

}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participant {
    id: Uuid,
    name: String,
    payer: bool,
    settle_status: bool
}

impl Participant {
    fn new(name: String) -> Self {
        Participant {
           id: Uuid::new_v4(),
           name,
           payer: false,
           settle_status: false
        }
    }

    fn update_name(&mut self, name: String) {
        self.name = name;
    }

    fn mark_as_payer(&mut self) {
        self.payer = true;
    }

    fn mark_as_paid(&mut self) {
        if self.settle_status == false {
            self.settle_status = true
        } else {
            // TODO: throw an error here, until then log an error
            //
            debug!("user has already paid!")
        }
    }


    fn mark_as_unpaid(&mut self) {
        if self.settle_status == true {
            self.settle_status = false
        } else {
            // TODO: throw an error here, until then log an error
            //
            debug!("user has already not-paid!")
        }
    }
}



#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitItem {
    id: Uuid,
    event_name: String,
    total_price: Decimal,
    total_tax: Decimal,
    participants: Vec<Participant>,
    line_items: Vec<LItem>,
    final_split: HashMap<Uuid, Decimal>,
    settle_status: bool
}


impl SplitItem {
    fn new (event_name: String, total_price: Decimal, total_tax: Decimal, participants: Vec<Participant>, line_items: Vec<LItem>) -> Self {
        SplitItem {
           id: Uuid::new_v4(),
           event_name,
           total_price,
           total_tax,
           participants,
           line_items,
           final_split: HashMap::new(),
           settle_status: false
        }
    }

    fn calculate_split(&mut self) {
        for part in self.participants.iter() {
            let current_part_split = self.line_items.iter().fold(Decimal::new(0,10), | mut acc, x|{
                if x.participants.get().iter().any(|p| p.id == part.id) {
                     acc += x.get_split_by_participants();
                     acc
                } else {
                    acc
            }
        });
        self.final_split.insert(part.id, current_part_split);
     }
    }

    fn get_split_amount_by_id(&self, id: Uuid) -> Option<Decimal> {
       self.final_split.get(&id).cloned()
    }
}




#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        // sets the document title
        <Title text="Whati8"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=Homepage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}



#[component]
fn Homepage() -> impl IntoView {

    let (participants, set_participants) = create_signal(Participants::new());

    let part_name_ref = create_node_ref::<Input>();
    let part_payer_ref_yes = create_node_ref::<Input>();
    let add_participant = move || {
        let name_input = part_name_ref.get().unwrap();
        let payer_input = part_payer_ref_yes.get().unwrap();
        let is_payer = payer_input.checked();
        let name = String::from(name_input.value().trim());
        if !name.is_empty() {
            let mut new = Participant::new(name.clone());
            if is_payer {
                new.mark_as_payer();
            }
            set_participants.update(|p| p.add(new.clone()));
            name_input.set_value("");
        }
    };


    let (litem, set_litem) = create_signal(LItems::new());

    let litem_name_ref = create_node_ref::<Input>();
    let litem_price_ref = create_node_ref::<Input>();
    let litem_parts_ref = create_node_ref::<Select>();
    let add_line_item = move || {
        let litem_name_input = litem_name_ref.get().unwrap();
        let litem_price_input = litem_price_ref.get().unwrap();
        let litem_parts_input = litem_parts_ref.get().unwrap();
        let name = String::from(litem_name_input.value().trim());
        let price = Decimal::from_str_exact(litem_price_input.value().as_str()).unwrap();
        let parts = litem_parts_input.selected_options();
        let mut parts_selected = Vec::new();
        for i in 0..parts.length() {
            if let Some(node) = parts.get_with_index(i) {
                if let Some(element) = node.dyn_ref::<web_sys::HtmlOptionElement>() {
                    parts_selected.push(element.value());
                }

            }
        }

        if !name.is_empty() && !price.to_string().is_empty() {
            let mut new = LItem::new(name.clone(), price);
            set_litem.update(|l| l.add(new.clone()));
            let participants_list: Vec<String> = participants.get().0.iter().map(|p| p.id.to_string()).collect();
            let selected_participants: Vec<Participant> = participants_list
            .iter()
            .filter(|&p| parts_selected.contains(p))
            .map(|p| participants.get().0.iter().find(|participant| participant.id.to_string() == *p).cloned().unwrap())
            .collect();

            for participant in selected_participants {
                new.add_participant(participant.clone());
            }
            litem_name_input.set_value("");
            litem_price_input.set_value("");
        }

    };


    let all_participants = move || {
        participants.get().0
    };

    let participants_exists = move || all_participants().len() > 0;


    create_effect(move |_| {
        if let Ok(Some(storage)) = window().local_storage() {
            let json = serde_json::to_string(&participants).expect("Couldn't serialize json");
            if storage.set_item(STORAGE_KEY_PARTICIPANTS, &json).is_err() {
                log::error!("Error while trying to set item in participants");
            }
        }
    });


    create_effect(move |_| {
        if let Ok(Some(storage)) = window().local_storage() {
            let json = serde_json::to_string(&litem).expect("Couldn't serialize json");
            if storage.set_item(STORAGE_KEY_LITEM, &json).is_err() {
                log::error!("Error while trying to set item in line items");
            }
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        add_participant()
    };

    let on_submit_item = move |ev: SubmitEvent| {
        ev.prevent_default();
        add_line_item()
    };


    let edit_participant_name = move |value: String, id: String| {
        set_participants.update(|participants| {
        if let Some(participant) = participants.0.iter_mut().find(|p| p.id.to_string() == id) {
            participant.name = value.clone();
        }
        });
    };






    view! {
        <body class="bg-gray-100">
            <main class="container mx-auto p-4">
                <section class="bg-white p-8 rounded-lg shadow-md max-w-md mx-auto mt-8">
                    <div class="mb-4">
                        <div>Whati8 phase-1</div>
                        <input type="text" id="e-name" placeholder="Enter the event name.." class="mt-2 p-2 border rounded-md w-full" />
                    </div>
                    <div id="add-participant-section" class=" mt-4">
                                <div class="w-1/2 pr-2">
                                    <label for="participant-dropdown">Participants</label>

                                    {move || if participants_exists()
                                        {
                                        view! {
                                          <div>
                                                <For
                                                    each=all_participants
                                                    key=|part| part.id
                                                    let:part
                                                >
                                                 <div class="flex items-center mb-2">
                                                         <input
                                                         on:input=move |ev| edit_participant_name(event_target_value(&ev), part.id.to_string())
                                                             type="text" value=part.name class="mr-2 border rounded-md p-2" />
                                                         <label class="mr-2">Payer</label>
                                                         <input type="checkbox"  name="is-payer" checked=move || part.payer == true class="mr-2" />
                                                         <button class="bg-red-500 text-white p-2 rounded-md">Remove</button>
                                                 </div>
                                                 </For>
                                          </div>
                                        }
                                    }else {
                                        view! {
                                          <div class="flex items-center mb-2">
                                              <h3>No participants yet!</h3>
                                          </div>

                                        }
                                    }
                                    }

                                </div>
                                <form on:submit=on_submit>
                                <label for="participant-name">Participant Name</label>
                                <input type="text" node_ref=part_name_ref id="participant-name" placeholder="Enter participant name" class="mt-2 p-2 border rounded-md w-full" />

                                <label class="mt-2">Is Payer?</label>
                                <div class="flex items-center">
                                    <input type="radio" node_ref=part_payer_ref_yes id="is-payer-yes" name="is-payer" value="yes" class="mr-2"/>
                                    <label for="is-payer-yes">Yes</label>

                                    <input type="radio" id="is-payer-no" checked name="is-payer" value="np" class="ml-4"/>
                                    <label for="is-payer-no">No</label>
                                </div>

                                <div class="w-1/2 pl-2">
                                <label for="add-participant-btn" class="invisible">Add Participant</label>
                                <button type="submit" id="add-participant-btn" class="mt-2 p-2 border rounded-md w-full bg-blue-500 text-white">Add Participant</button>
                                </div>
                                </form>
                                <form on:submit=on_submit_item>
                                 <label for="item-name" class="mt-2">Item Name</label>
                                 <input type="text" node_ref=litem_name_ref id="item-name" placeholder="Enter item name" class="mt-2 p-2 border rounded-md w-full"/>

                                 <label for="item-price" class="mt-2">Item Price</label>
                                 <input type="number" node_ref=litem_price_ref id="item-price" placeholder="Enter item price" class="mt-2 p-2 border rounded-md w-full" step="0.01"/>

                                 <label for="participants-dropdown" class="mt-2">Select Participants</label>
                                 <select id="participants-dropdown" node_ref=litem_parts_ref multiple class="mt-2 p-2 border rounded-md w-full">
                                     <For each=all_participants key=|part| part.id let:part>
                                         <option value=part.clone().id.to_string()>{part.clone().name}</option>
                                     </For>
                                 </select>

                                 <div class="w-1/2 pl-2 mt-2">
                                     <button type="submit" id="add-item-btn" class="p-2 border rounded-md w-full bg-blue-500 text-white">Add Item</button>
                                 </div>
                                 </form>



                    </div>
                </section>
            </main>
        </body>
    }
}


/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
