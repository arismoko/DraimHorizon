//! Camera zoom on the typed API: while the `zoom` key is held in play, the view zooms by the
//! chosen factor, and the mouse wheel steps through the factors instead of the hotbar.
use mod_api::{ClientHost, LivingRef, ModKeyMappingRef};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The mod's `sunrise-mod.toml` id.
const ID: &str = "draimhorizon";
pub const DEFAULT_KEY: &str = "key.keyboard.z";
const FACTORS: [f32; 8] = [2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0];
static LEVEL: AtomicUsize = AtomicUsize::new(3);

thread_local! {
    /// The `zoom` mapping's handle, resolved again when a new registry generation refuses it.
    static KEY: Cell<Option<ModKeyMappingRef>> = const { Cell::new(None) };
}

fn key_down(host: &dyn ClientHost) -> bool {
    KEY.with(|key| {
        if let Some(mapping) = key.get()
            && let Ok(down) = mapping.is_down(host)
        {
            return down;
        }
        let Ok(mapping) = host.mod_key_mapping(ID, "zoom") else {
            return false;
        };
        key.set(Some(mapping));
        mapping.is_down(host).unwrap_or(false)
    })
}

/// The local player as a living handle.
fn local_living(host: &dyn ClientHost) -> Option<LivingRef> {
    host.local_player()
        .and_then(|player| host.living(player.entity()))
}

/// In play with nothing over the world, the player awake, the mod on and no `zoom` mod on to
/// take the key, and the key held. The game itself fires zoom only in play and not while
/// looking through a spyglass.
fn held(host: &dyn ClientHost) -> bool {
    host.on(ID)
        && !(host.is_loaded("zoom") && host.on("zoom"))
        && host.window_focused()
        && host.playing()
        && host.observed_screen_id().is_none()
        && local_living(host).is_some_and(|living| living.is_sleeping(host) != Some(true))
        && key_down(host)
}

/// The camera player looks through a spyglass in first person (`AbstractClientPlayer.isScoping`):
/// the game then fixes the modifier and skips `compute_fov_modifier`.
fn scoping(host: &dyn ClientHost) -> bool {
    let Some(spyglass) = mod_api::Identifier::parse("minecraft:spyglass")
        .ok()
        .and_then(|id| host.item_key(&id))
    else {
        return false;
    };
    host.camera_type().is_first_person()
        && host.local_item_use().is_some_and(|using| {
            local_living(host).is_some_and(|living| {
                living
                    .with_hand_item(host, using.hand, |stack| stack.item() == spyglass)
                    .unwrap_or(false)
            })
        })
}

/// The zoom of this frame's view as a divisor of the field-of-view modifier, or none: the key held
/// and nothing over the world. The game asks no `compute_fov_modifier` listener while the camera
/// player looks through a spyglass, so the mod's zoom never stacks on the scope.
pub fn factor(host: &dyn ClientHost) -> Option<f32> {
    (held(host) && !scoping(host)).then(|| FACTORS[LEVEL.load(Ordering::Relaxed)])
}

/// Vanilla's camera eases to the modifier this leaves, as it did for the old smooth zoom.
pub fn compute_fov_modifier(host: &dyn ClientHost, modifier: &mut f32) {
    if let Some(factor) = factor(host) {
        *modifier /= factor;
    }
}

/// Mouse, stick and key turning slows by the same factor while zoomed.
pub fn player_turn(host: &dyn ClientHost, x: &mut f64, y: &mut f64) {
    if let Some(factor) = factor(host) {
        let scale = f64::from(factor.recip());
        *x *= scale;
        *y *= scale;
    }
}

/// Steps the factor by the wheel's whole steps; true when the wheel was used for zoom.
/// Only while the zoom is in effect: while looking through a spyglass the wheel stays the hotbar's.
pub fn scroll(host: &dyn ClientHost, steps: i32) -> bool {
    if !held(host) || scoping(host) {
        return false;
    }
    let level = LEVEL.load(Ordering::Relaxed);
    LEVEL.store(
        level
            .saturating_add_signed(steps as isize)
            .min(FACTORS.len() - 1),
        Ordering::Relaxed,
    );
    true
}
