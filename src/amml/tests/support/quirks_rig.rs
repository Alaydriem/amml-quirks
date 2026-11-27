use alaydriem_quirks::QuirksMod;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use amml_api::Area;
use amml_api::amml_abi::{AmmlRouteVia, ClientTick, IdleScreenOpen};
use amml_api::testing::{FakeActivity, FakeClient, FakeGame};

// quirks enabled on a fake game that serves every event and area of its `USES`
pub struct QuirksRig {
    pub game: FakeGame,
}

impl QuirksRig {
    // the mod manager is provided with `config` as the mod's config.hcl, and every mod has loaded
    pub fn with_manager(config: &str) -> Self {
        let rig = Self::build(true, config);
        FakeTable::mods_loaded(&rig.game);
        FakeTable::ready(&rig.game);
        rig
    }

    // the mod manager is not installed, and every mod has loaded
    pub fn without_manager() -> Self {
        let rig = Self::build(false, "");
        FakeTable::mods_loaded(&rig.game);
        rig
    }

    fn build(manager: bool, config: &str) -> Self {
        let game = FakeTable::game();
        game.offer::<ClientTick>();
        game.offer::<IdleScreenOpen>();
        game.offer_area(Area::CLIENT_ACTIVITY);
        game.offer_area(Area::INPUT_LEASE);
        if manager {
            FakeTable::provide(&game.ctx()).unwrap();
        }
        FakeTable::set_config(config);
        game.load::<QuirksMod>().unwrap();
        game.enable::<QuirksMod>().unwrap();
        game.open_gate();
        let rig = Self { game };
        rig.activity(FakeActivity::Ok);
        rig
    }

    // what `note_activity` gives from now on
    pub fn activity(&self, activity: FakeActivity) {
        self.game.set_client(FakeClient { activity, ..FakeClient::default() });
    }

    pub fn tick(&self) {
        self.game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    }

    // whether the game's ask for the idle screen was cancelled
    pub fn idle_screen(&self, via: AmmlRouteVia) -> bool {
        let mut event = IdleScreenOpen { header: FakeGame::header::<IdleScreenOpen>(), via, _reserved: 0 };
        self.game.raise_with_client(&mut event).cancelled
    }

    pub fn manager_event(&self, event: ModEvent) {
        FakeTable::push_event(event);
        self.tick();
    }

    // the number of `note_activity` calls that reached the game
    pub fn activity_calls(&self) -> usize {
        self.game.client_writes().iter().filter(|write| *write == "note_activity").count()
    }

    // the manager is being disabled: its tables go, and the link passes `ManagerGone` on
    pub fn manager_gone(&self) {
        FakeTable::gone(&self.game);
        self.tick();
    }

    // true when one log line holds both parts
    pub fn logged_with(&self, first: &str, second: &str) -> bool {
        self.game.logs().iter().any(|line| line.contains(first) && line.contains(second))
    }

    pub fn logged(&self, part: &str) -> bool {
        self.game.logs().iter().any(|line| line.contains(part))
    }
}
