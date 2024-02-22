use std::{collections::HashMap, string};
use leptos::{ev::SubmitEvent, html::Input, *};
use leptos_meta::*;
use leptos_router::*;
use log::{info, debug};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::{prelude::ToPrimitive, Decimal};


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LItem {
    id: Uuid,
    item_name: String,
    price: Decimal,
    participants: Vec<Participant>,
}


impl LItem {
    fn new(item_name: String, price: Decimal) -> Self {
        LItem {
            id: Uuid::new_v4(),
            item_name,
            price,
            participants: Vec::new()
        }
    }

    fn add_participant(&mut self, participant: Participant)  {
        self.participants.push(participant);
    }

    fn remove_participant(&mut self, participant_id: Uuid) {
        self.participants.retain(|x| x.id != participant_id)
    }

    fn add_bulk_participants(&mut self, participants: Vec<Participant>) {
        self.participants = participants;
    }
}



#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Participant {
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
struct SplitItem {
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
        let mut current_split : HashMap<Uuid, Decimal> = HashMap::new();
        for part in self.participants.iter() {
            let current_part_split = self.line_items.iter().fold(Decimal::new(0,10), | mut acc, x|{
                if x.participants.contains(&part) {
                     acc += x.price / Decimal::from(x.participants.len());
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
    view! {
        <body class="bg-gray-100">
            <main class="container mx-auto p-4">
                <section class="bg-white p-8 rounded-lg shadow-md max-w-md mx-auto mt-8">
                    <div class="mb-4">
                        <div>Whati8 phase-1</div>
                        <input type="text" id="e-name" placeholder="Enter the event name.." class="mt-2 p-2 border rounded-md w-full" />
                    </div>
                    <label for="e-date">Enter event date</label>
                    <input type="date" id="e-date" class="mt-2 p-2 border rounded-md w-full" />
                    <label for="e-tax">Enter tax amount</label>
                    <input type="double" id="e-tax" placeholder="Enter the tax amount" class="mt-2 p-2 border rounded-md w-full" />
                    <label for="e-amount">Enter total amount</label>
                    <input type="double" id="e-amount" placeholder="Enter the total amount" class="mt-2 p-2 border rounded-md w-full" />
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
