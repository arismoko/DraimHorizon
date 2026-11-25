fn draimhorizons_inventory_search(
    game: &crate::game::Game<'_>,
    stack: &sunrise_client::items::ItemStack,
    x: f32,
    y: f32,
    draw: &mut dyn mod_api::GuiCanvas,
) {
    // The decoration hook hands the mod a typed canvas, not the HUD batch.
    if let Some((rects, color)) =
        crate::game::claim_borders_inventory_search_view::highlight(game, stack, x, y)
    {
        for rect in rects {
            let _ = draw.fill(rect, color);
        }
    }
}
