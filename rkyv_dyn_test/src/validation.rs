#[cfg(test)]
mod tests {
    use bytecheck::CheckBytes;
    use rkyv::{
        check_archived_root,
        ser::{serializers::AlignedSerializer, Serializer},
        util::AlignedVec,
        validation::DefaultArchiveValidator,
        Archive, Archived, Serialize,
    };
    use rkyv_dyn::archive_dyn;
    use rkyv_typename::TypeName;

    fn serialize_and_check<T: Serialize<AlignedSerializer<AlignedVec>>>(value: &T)
    where
        T::Archived: CheckBytes<DefaultArchiveValidator>,
    {
        let mut serializer = AlignedSerializer::new(AlignedVec::new());
        serializer
            .serialize_value(value)
            .expect("failed to archive value");
        let buf = serializer.into_inner();
        check_archived_root::<T>(buf.as_ref()).unwrap();
    }

    #[test]
    #[cfg(not(feature = "wasm"))]
    fn check_dyn() {
        #[archive_dyn]
        pub trait TestTrait {
            fn get_id(&self) -> i32;
        }

        #[derive(Archive, Serialize)]
        #[archive_attr(derive(CheckBytes, TypeName))]
        pub struct Test {
            id: i32,
        }

        #[archive_dyn]
        impl TestTrait for Test {
            fn get_id(&self) -> i32 {
                self.id
            }
        }

        impl TestTrait for Archived<Test> {
            fn get_id(&self) -> i32 {
                self.id.into()
            }
        }

        let value: Box<dyn SerializeTestTrait> = Box::new(Test { id: 42 });

        serialize_and_check(&value);

        #[derive(Archive, Serialize)]
        #[archive_attr(derive(TypeName))]
        pub struct TestUnchecked {
            id: i32,
        }

        #[archive_dyn]
        impl TestTrait for TestUnchecked {
            fn get_id(&self) -> i32 {
                self.id
            }
        }

        impl TestTrait for Archived<TestUnchecked> {
            fn get_id(&self) -> i32 {
                self.id.into()
            }
        }

        let value: Box<dyn SerializeTestTrait> = Box::new(TestUnchecked { id: 42 });

        let mut serializer = AlignedSerializer::new(AlignedVec::new());
        serializer
            .serialize_value(&value)
            .expect("failed to archive value");
        let buf = serializer.into_inner();
        if let Ok(_) = check_archived_root::<Box<dyn SerializeTestTrait>>(buf.as_ref()) {
            panic!("check passed for type that does not implement CheckBytes");
        }
    }
}