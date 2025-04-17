////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Helper macro for extracting multiple components at once.
///
/// # Examples
///
/// ```ignore
/// // Most often, we extract only one component:
/// let current_account_subject = dill::from_catalog_n!(catalog, CurrentAccountSubject);
///
/// // But sometimes, three at once:
/// let (dataset_registry, polling_ingest_svc, dataset_changes_svc) = dill::from_catalog_n!(
///     catalog,
///     dyn DatasetRegistry,
///     dyn PollingIngestService,
///     dyn DatasetChangesService
/// );
#[macro_export]
macro_rules! from_catalog_n {
    ($catalog:ident, $T:ty) => {{
        $catalog.get_one::<$T>().unwrap()
    }};
    ($catalog:ident, $T:ty, $($Ts:ty),+) => {{
        ( $catalog.get_one::<$T>().unwrap(), $( $catalog.get_one::<$Ts>().unwrap() ),+ )
    }};
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
