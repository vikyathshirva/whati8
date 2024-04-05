use std::{collections::HashMap, ops::{Add, AddAssign}, str::FromStr, string, thread::current};
use leptos::{ev::SubmitEvent, html::{Input, Select}, *};
use leptos_meta::*;
use leptos_router::*;
use log::{info, debug, LevelFilter};
use serde::{Deserialize, Serialize};
use uuid::{timestamp::UUID_TICKS_BETWEEN_EPOCHS, Uuid};
use rust_decimal::{prelude::{FromPrimitive, ToPrimitive}, Decimal};
use wasm_bindgen::JsCast;
use web_sys::{js_sys::{Date, Intl::DateTimeFormat}, Blob};
use rusty_money::{Money, iso, Locale};

const STORAGE_KEY_LITEM : &str = "litems-key";
const STORAGE_KEY_PARTICIPANTS : &str = "participants-key";
const STORAGE_KEY_SPLIT_ITEM: &str = "split-item-key";


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LItems(pub Vec<LItem>);
impl LItems {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, litem: LItem) {
        self.0.push(litem);
    }
    pub fn remove(&mut self, id: String) {
        self.0.retain(|p| p.id.to_string() != id);
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

    fn update_name(&self, updated_name: String) {
        self.item_name.update(|item_name| *item_name = updated_name.clone());
    }

    fn clear_participants(&self) {
        self.participants.update(|p| p.retain(|x| x.id.to_string() == "dummy"));
    }


    fn update_price(&self, updated_price: Decimal) {
        self.price.update(|up| *up = updated_price.round_dp(2));
    }

    fn get_split_by_participants(&self) -> Decimal {
        self.price.get() / Decimal::from(self.participants.get().len())
    }

    fn add_participant(&mut self, participant: Participant)  {
        self.participants.update(|p| p.push(participant))
    }

    fn remove_participant(&mut self, participant_id: String) {
        self.participants.update(|p| p.retain(|x| x.id.to_string() != participant_id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participants(pub Vec<Participant>);

impl Participants {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, participant: Participant) {
        self.0.push(participant);
    }

    pub fn remove(&mut self, id: String) {
        self.0.retain(|p| p.id.to_string() != id);
    }

}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participant {
    id: Uuid,
    name: RwSignal<String>,
    payer: RwSignal<bool>,
    settle_status: RwSignal<bool>
}

impl Participant {
    fn new(name: String) -> Self {
        let name = create_rw_signal(name);
        let payer = create_rw_signal(false);
        let settle_status = create_rw_signal(false);
        Participant {
           id: Uuid::new_v4(),
           name,
           payer,
           settle_status
        }
    }

    fn is_payer(&self) ->bool {
        self.payer.get()
    }

    fn update_name(&mut self, new_name: String) {
        self.name.update(|name| *name = new_name.clone());
    }

    fn mark_as_payer(&mut self) {
        self.payer.update(|payed| *payed = !*payed);
    }

    fn mark_as_paid(&mut self) {
        if self.settle_status.get() == false {
        self.settle_status.update(|payed| *payed = !*payed);
        } else {
            // TODO: throw an error here, until then log an error
            //
            debug!("user has already paid!")
        }
    }


    fn mark_as_unpaid(&mut self) {
        if self.settle_status.get() == true {
        self.settle_status.update(|payed| *payed = !*payed);
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
    event_name: RwSignal<String>,
    total_price: RwSignal<Decimal>,
    total_tax: RwSignal<Decimal>,
    participants: RwSignal<Vec<Participant>>,
    line_items: RwSignal<Vec<LItem>>,
    final_split: RwSignal<HashMap<Uuid, Decimal>>,
    settle_status: RwSignal<bool>,
    summary_text: RwSignal<String>
}


impl SplitItem {
    fn new () -> Self {
        let event_name = create_rw_signal(String::new());
        let total_price = create_rw_signal(Decimal::new(0,2));
        let total_tax = create_rw_signal(Decimal::new(0,2));
        let participants = create_rw_signal(Vec::new());
        let line_items = create_rw_signal(Vec::new());
        let final_split = create_rw_signal(HashMap::new());
        let settle_status = create_rw_signal(false);
        let summary_text = create_rw_signal(String::new());
        SplitItem {
           id: Uuid::new_v4(),
           event_name,
           total_price,
           total_tax,
           participants,
           line_items,
           final_split,
           settle_status,
           summary_text
        }
    }

    fn add_event_name(&mut self, event_name: String) {
        self.event_name.update(|e| *e = event_name);
    }

    fn add_total_tax(&mut self, tax: Decimal) {
        self.total_tax.update(|p| *p = tax.round_dp(2));
    }


    fn add_participant(&mut self, participant: Participant) {
        self.participants.update(|p| p.push(participant));
    }


    fn add_line_item(&mut self, li: LItem) {
        self.line_items.update(|p| p.push(li));
    }

    fn remove_line_item(&mut self, id: String) {
        self.line_items.update(|p| p.retain(|li| li.id.to_string() != id));
    }

    fn remove_final_split_item(&mut self, id: Uuid) {
        self.final_split.update(|p| {
            p.remove(&id);
            *p = p.clone()
        });
    }

    fn remove_participant(&mut self, id: String) {
        self.participants.update(|p| p.retain(|p| p.id.to_string() != id));
    }

    fn total_involved_participant_count(&mut self) -> Decimal {
        let mut count = Decimal::new(0, 0); // Change 2 to 0 for integer part
        for part in &self.participants.get() {
            if self.is_involved_in_any_orders(part) {
                count += Decimal::new(1, 0); // Increment count by 1
            }
        }

        count
    }


    fn tax_split(&mut self) -> Decimal {
        if self.total_tax.get().ceil() > Decimal::from(0) && Decimal::from(self.final_split.get().len()) > Decimal::from(0) && self.total_involved_participant_count() > Decimal::from(0) {
            self.total_tax.get().round_dp(2) / self.total_involved_participant_count()
        } else {
            Decimal::from(0)
        }
    }

    fn is_involved_in_any_orders(&mut self, part: &Participant) -> bool {
       for litem in self.line_items.get().iter() {
        if litem.participants.get().contains(part) {
            return true;
        }
       } 
       return false;
    }

    pub fn update_summary_text(&self) {
        let event_name = self.event_name.get().clone();
        let total_price = self.total_price.get();
        let total_tax = self.total_tax.get();
        let participants = self.participants.get();
        let line_items = self.line_items.get();
        let final_split = self.final_split.get();

        let mut summary_text = String::new();
        summary_text.push_str(&format!("-----BILL SPLIT SUMMARY (COPY THIS)---- \n"));
        summary_text.push_str(&format!("Event Name: {}\n", event_name));
        summary_text.push_str(&format!("Total Price: ₹{}\n", total_price));
        summary_text.push_str(&format!("Total Tax: ₹{}\n\n", total_tax));

        summary_text.push_str("Participants:\n");
        for participant in participants {
            summary_text.push_str(&format!("- Name: {} ", &participant.name.get()));
            if participant.payer.get() {
                summary_text.push_str(" [PAYER] \n");
            }else {
                 summary_text.push_str("\n");
            }
            summary_text.push_str("  Items:\n");
            for item in line_items.iter() {
                if item.participants.get().clone().iter().any(|p| p.id == participant.id) {
                    summary_text.push_str(&format!("    - {}: ₹{}\n", &item.item_name.get(), &item.price.get()));
                }
            }
            summary_text.push_str(&format!("  Total Amount Owed: ₹{}\n\n", final_split.get(&participant.id).unwrap_or(&Decimal::ZERO)));
        }

        self.summary_text.set(summary_text);
    }



    fn calculate_split(&mut self) {
        for part in self.participants.get().iter() {
        let current_part_split = self.line_items.get().iter().fold(
            Decimal::new(0, 2),
            |mut acc, x| {
                if x.participants.get().iter().any(|p| p.id == part.id) {
                    acc += x.get_split_by_participants().round_dp(2);
                }
                acc.round_dp(2)
            },
        );
        let tax = self.tax_split();
        if self.is_involved_in_any_orders(part) {
            self.final_split.update(|s| {
                s.insert(part.id, Decimal::add(current_part_split, tax.round_dp(2)));
            });
        } else {
            self.final_split.update(|s| {
                s.insert(part.id, Decimal::from(0).round_dp(2));
            });
        }

        let mut total = Decimal::new(0,0);
        for (_id, &amt) in self.final_split.get().iter() {
            total = Decimal::add(total, amt.round_dp(2));
        }
        self.total_price.update(|p| *p = total.round_dp(2));
        self.update_summary_text()
        } 
    }

    fn get_split_amount_by_id(&self, id: Uuid) -> Option<Decimal> {
       self.final_split.get().get(&id).cloned()
    }
}




#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Link rel="shortcut icon" type_="image/ico" href="./assets/favicon.ico"/>
        <link rel="icon" type="image/png" sizes="192x192" href="./assets/android-chrome-192x192.png"/>
        <link rel="icon" type="image/png" sizes="512x512" href="./assets/android-chrome-512x512.png"/>
        <link rel="apple-touch-icon" href="./assets/apple-touch-icon.png"/>
        <link rel="icon" type="image/png" sizes="16x16" href="./assets/favicon-16x16.png"/>
        <link rel="icon" type="image/png" sizes="32x32" href="./assets/favicon-32x32.png"/>

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
    let (split_item, set_split_item) = create_signal(SplitItem::new());
    let mark_only_one_payer = move |id: String| {
        participants
        .get()
        .0
        .iter_mut()
        .filter(|p| p.id.to_string() != id && p.payer.get() == true)
        .for_each(|p| p.mark_as_payer());
        set_participants.update(|participants|{});
    };

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
            split_item.get().add_participant(new.clone());
            split_item.get().calculate_split();
            if is_payer {
               mark_only_one_payer(new.id.to_string().clone());
            }
            name_input.set_value("");
        }
    };






    


    let (litems, set_litems) = create_signal(LItems::new());
    let remove_participant = move |id: String| {
    if !id.is_empty() {
            set_participants.update(|participants| {
                participants.remove(id.clone());
            });
            let mut litems_clone = Vec::new();
            set_litems.update(|litems| {
                litems_clone = litems.0.clone();
                litems.0.clear();
            });
            for li in &mut litems_clone {
                li.remove_participant(id.clone());
            }
            set_litems.update(|litems| {
                *litems = LItems(litems_clone);
            });

            split_item.get().remove_participant(id.clone());
            split_item.get().remove_final_split_item(Uuid::from_str(id.as_str()).unwrap());
            split_item.get().calculate_split();
        }
    };


    



    let remove_line_item = move |id: String| {
        if !id.is_empty() {
            set_litems.update(|litems| {
                litems.remove(id.clone());
            });
            split_item.get().remove_line_item(id.clone());
            split_item.get().calculate_split();
        }
    };


    let litem_name_ref = create_node_ref::<Input>();
    let litem_price_ref = create_node_ref::<Input>();
    let litem_parts_ref = create_node_ref::<Select>();
    let split_item_ref_name = create_node_ref::<Input>();
    let split_item_total_price = create_node_ref::<Input>();
    let split_item_total_tax = create_node_ref::<Input>();


    let update_split_event_name = move |name: String| {
        let mut split_i = split_item.get();
        split_i.add_event_name(name);

    };


    let update_split_event_total_tax = move |value: Decimal| {
        let mut split_i = split_item.get();
        split_i.add_total_tax(value);
        split_item.get().calculate_split();
    };



    let mut add_line_item = move || {
        let litem_name_input = litem_name_ref.get().unwrap();
        let litem_price_input = litem_price_ref.get().unwrap();
        let litem_parts_input = litem_parts_ref.get().unwrap();
        let name = String::from(litem_name_input.value().trim());
        let price = Decimal::from_str_exact(litem_price_input.value().as_str()).unwrap_or(Decimal::from(0));
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
            set_litems.update(|l| l.add(new.clone()));
            let participants_list: Vec<String> = participants.get().0.iter().map(|p| p.id.to_string()).collect();
            let selected_participants: Vec<Participant> = participants_list
            .iter()
            .filter(|&p| parts_selected.contains(p))
            .map(|p| participants.get().0.iter().find(|participant| participant.id.to_string() == *p).cloned().unwrap())
            .collect();

            for participant in selected_participants {
                new.add_participant(participant.clone());
                parts_selected.clear();
            }
            split_item.get().add_line_item(new);
            split_item.get().calculate_split();
            litem_name_input.set_value("");
            litem_price_input.set_value("");
        }

    };


    let all_participants = move || {
        participants.get().0
    };

    let participants_exists = move || all_participants().len() > 0;


    let all_litems = move || {
        litems.get().0
    };

    let litems_exists = move || all_litems().len() > 0;


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
            let json = serde_json::to_string(&split_item).expect("Couldn't serialize json");
            if storage.set_item(STORAGE_KEY_SPLIT_ITEM, &json).is_err() {
                log::error!("Error while trying to set item in participants");
            }
        }
    });

    create_effect(move |_| {
        if let Ok(Some(storage)) = window().local_storage() {
            let json = serde_json::to_string(&litems).expect("Couldn't serialize json");
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
        let mut part = participants.get().0.iter().find(|p| p.id.to_string()== id).unwrap().clone();
        part.update_name(value);
        split_item.get().calculate_split();
    };


    let edit_item_name = move |value: String, id: String| {
        let litem = litems.get().0.iter().find(|l| l.id.to_string() == id).unwrap().clone();
        litem.update_name(value);
        split_item.get().calculate_split();
    };


    let edit_item_price = move |value: String, id: String| {
        let litem = litems.get().0.iter().find(|l| l.id.to_string() == id).unwrap().clone();
        litem.update_price(Decimal::from_str_exact(value.as_str()).unwrap().clone());
        split_item.get().calculate_split();
    };


    let edit_participant_payer = move |id: String| {
        let mut part = participants.get().0.iter().find(|p| p.id.to_string()== id).unwrap().clone();
        part.mark_as_payer();
        mark_only_one_payer(id.clone());
        split_item.get().calculate_split();
    };



    let edit_selected_parts = move |id: String, event: web_sys::Event| {
    let mut litem = litems.get().0.iter().find(|l| l.id.to_string() == id).unwrap().clone();
    if let Some(target) = event.target() {
        if let Some(parts_input) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            let mut parts_selected = Vec::new();

            let parts = parts_input.selected_options();

            for i in 0..parts.length() {
                if let Some(node) = parts.get_with_index(i) {
                    if let Some(element) = node.dyn_ref::<web_sys::HtmlOptionElement>() {
                        parts_selected.push(element.value());
                    }
                }
            }

            litem.clear_participants();

            for participant in parts_selected {
                if let Some(part) = participants.get().0.iter().find(|p| p.id.to_string() == participant) {
                    litem.add_participant(part.clone());
                }
            }
            split_item.get().calculate_split();
        }
    }
    };


    let check_if_litem_participant = move |part_id: String, litem_id: String| {
        if let Some(litem) = litems.get().0.iter().find(|l| l.id.to_string() == litem_id) {
            litem.participants.get().iter().any(|p| p.id.to_string() == part_id)
        } else {
            false
        }
    };





    view! {
        <body class="bg-gray-100">
            <main class="container mx-auto p-4">
                <section class="bg-white p-8 rounded-lg shadow-md max-w-md mx-auto mt-8">
                    <div class="mb-4">
                        <img src="./assets/headline.png"/>
                       <div style="text-align: center; padding-top: 20px; padding-bottom: 20px;">
                            A simple, no-nonsense
                            <span style="color: pink; font-style: italic;">bill splitting </span>
                            app
                        </div>
                        <label for="e-name">Event name</label>
                        <input type="text" id="e-name"
                        on:input=move |ev| update_split_event_name(event_target_value(&ev))
                        placeholder="Enter the event name.." class="mt-2 p-2 border rounded-md w-full" />
                    </div>
                    <div id="calculate-split-section" class=" mt-4">
                                    <label for="participant-dropdown">Item Split</label>
                                    {move || if litems_exists()
                                        {
                                        view! {
                                          <div>
                                            <form on:submit=on_submit>
                                            <div class="border-dashed border-2 border-pink-500 mt-4 p-4 rounded-md ">
                                                 <label for="total-amount">Total Amount</label>
                                                 <input type="text"  id="total-amount" placeholder="Total Amount"
                                                value=split_item.get().total_price.get().to_string()
                                                class="mt-2 p-2 border rounded-md w-full" readonly/>
                                            </div>
                                                     <div class="space-y-2">
                                                         <For each=all_participants key=|part| part.id let:part>
                                                         {move || if part.is_payer() {

                                                                view! {
                                                                    <div class="border-dotted border-2 border-green-500 mt-4 p-4 rounded-md ">
                                                                        <label for="payer" class="bg-green-400 rounded px-2 py-1">pay to</label>
                                                                        <div id="payer" class="flex items-center">
                                                                        <label for="participant-amount" class="w-1/2">{part.name} "'s share" </label>
                                                                        <input type="text"
                                                                        value=split_item.get().final_split.get().get(&part.id).unwrap_or(&Decimal::from(0)).to_string()

                                                                        id=format!("participant-amount_{}", part.id) placeholder="Enter amount" class="w-full mt-2 p-2 border rounded" readonly/>
                                                                        </div>
                                                                    </div>
                                                                }
                                                          } else {
                                                              view! {
                                                                  <div class="flex items-center">
                                                                     <label for="participant-amount" class="w-1/2">{part.name} "'s share" </label>
                                                                     <input type="text"
                                                                     value=split_item.get().final_split.get().get(&part.id).unwrap_or(&Decimal::from(0)).to_string()
                                                                     id=format!("participant-amount_{}", part.id) placeholder="Enter amount" class="w-full mt-2 p-2 border rounded" readonly/>
                                                                 </div>
                                                              }
                                                         }
                                                         }
                                                         </For>
                                                     </div>
                                            </form> 
                                              <div class="space-y-2">
                                               <textarea id="summary" class="w-full h-full mt-4 p-2 border rounded" rows="20" readonly>{split_item.get().summary_text.get().clone()}</textarea>
                                                </div>
                                          </div>
                                        }
                                    }else {
                                        view! {
                                          <div class="flex items-center mb-2">
                                              <h3>No items to split yet!</h3>
                                          </div>

                                        }
                                    }
                                    }

                    </div>
                    <div id="add-participant-section" class=" mt-4">
                                <div class="w-full pr-2">
                                    <label for="participant-dropdown">Participants</label>

                                    {move || if participants_exists()

                                        {
                                        view! {
                                          <div >
                                                <For
                                                    each=all_participants
                                                    key=|part| part.id
                                                    let:part
                                                >
                                                 <div class="border-dashed border-2 border-pink-500 mt-4 p-4 rounded-md flex flex-col items-start mb-2">
                                                    <div class="flex items-center mb-2">
                                                         <input
                                                         on:input=move |ev| {
                                                             edit_participant_name(event_target_value(&ev), part.id.to_string())
                                                         }
                                                             type="text" value=part.name
                                                                 class="mb-2 border rounded-md p-2 w-full sm:w-auto" />
                                                        <label class="mb-2 flex-grow ml-2">Payer</label>
                                                             <div class="flex items-center">
                                                                 <input
                                                                     type="checkbox"
                                                                     on:click=move |_| edit_participant_payer(part.id.to_string())
                                                                     name="is-payer"
                                                                     checked=part.is_payer()
                                                                     class="mb-2 p-2 ml-2"
                                                                 />
                                                                 <button
                                                                     on:click=move |_| remove_participant(part.id.to_string())
                                                                     class="bg-red-500 text-white p-2 rounded-md ml-2 sm:ml-2"
                                                                 >
                                                                     Remove
                                                                 </button>
                                                             </div>
                                                    </div>
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


                    </div>

                    <div id="add-participant-section"  class="mt-4">
                                <div >
                                    <label for="participant-dropdown">{move || litems.get().0.len()} items </label>
                                    {move || if litems_exists()

                                    {
                                        view!{
                                            <>
                                                <For
                                                    each=all_litems
                                                    key=|litem| litem.id
                                                    let: litem
                                                >
                                                <div class="border-dotted border-2 border-blue-500 mt-4 p-4 rounded-md">

                                                <div class="w-1/2 pr-2 ">

                                                <div class="mb-2">
                                                    <label for="item-name">Item Name</label>
                                                    <input
                                                         on:input=move |ev| {
                                                             edit_item_name(event_target_value(&ev), litem.id.to_string())
                                                         }
                                                    type="text" id="item-name" value=litem.item_name placeholder="Enter item name" class="mr-2 border rounded-md p-2" />
                                                </div>
                                                <div class="mb-2">
                                                    <label for="item-price">Item Price</label>
                                                    <input
                                                         on:input=move |ev| {
                                                             edit_item_price(event_target_value(&ev), litem.id.to_string())
                                                         }
                                                    type="number" id="item-price" value=litem.price.get().to_string()
                                                    placeholder="Enter item price" class="mr-2 border rounded-md p-2" />

                                                </div>
                                                </div>
                                                {move ||
                                                    if participants_exists() {
                                                view! {
                                                    <div>
                                                        <label for="participant-dropdown">Participants</label>
                                                     <select id="participants-dropdown"
                                                     on:change=move |event: web_sys::Event| edit_selected_parts(litem.id.to_string(), event)
                                                     multiple class="mt-2 p-2 border rounded-md w-full">
                                                          <For each=all_participants key=|part| part.id let:part>
                                                          {move || if check_if_litem_participant(part.id.to_string(), litem.id.to_string()) {
                                                                view! {
                                                                    <option value=part.clone().id.to_string() selected >{part.clone().name}</option>
                                                                }
                                                                        } else {
                                                                view! {
                                                                    <option value=part.clone().id.to_string() >{part.clone().name}</option>
                                                                }
                                                          }
                                                        }
                                                          </For>
                                                     </select>
                                                    </div>
                                                      }
                                                    } else {
                                                        view! {
                                                            <div class="flex items-center mb-2">
                                                                <h3>No participants yet!</h3>
                                                            </div>
                                                            }
                                                        }
                                                      }
                                                <div >
                                                    <label for="rem-item-btn" class="sr-only">Remove Item</label>
                                                    <button on:click=move|_|remove_line_item(litem.id.to_string()) id="rem-item-btn" class="mt-2 p-2 border rounded-md w-full bg-red-500 text-white">
                                                        Remove Item
                                                    </button>
                                                </div>
                                                </div>
                                                </For>
                                            </>
                                        }
                                        }else {
                                            view! {
                                                <>
                                                    <h3>No Items yet!</h3>
                                                </>
                                            }
                                          }
                                        }

                                </div>
                    </div>


                    <div id="add-item-section" class=" mt-4">
                                <div>

                                    <label for="total-tax">Total tax</label>
                                    <input type="number"  id="total-amount"
                                    value={move || split_item.get().total_tax.get().to_string()}
                                    on:input=move |ev| {
                                        update_split_event_total_tax(Decimal::from_str_exact(event_target_value(&ev).as_str()).unwrap_or_else(|_| Decimal::from(0)));
                                     }
                                    id="Total Tax" class="mt-2 p-2 border rounded-md w-full" step="0.1"/>
                                </div>
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