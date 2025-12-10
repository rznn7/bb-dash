# Branch

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**links** | Option<[**serde_json::Value**](.md)> |  | [optional]
**name** | Option<**String**> | The name of the ref. | [optional]
**target** | Option<[**models::Commit**](commit.md)> |  | [optional]
**merge_strategies** | Option<**Vec<String>**> | Available merge strategies for pull requests targeting this branch. | [optional]
**default_merge_strategy** | Option<**String**> | The default merge strategy for pull requests targeting this branch. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


