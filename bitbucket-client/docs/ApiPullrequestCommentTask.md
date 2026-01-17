# ApiPullrequestCommentTask

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**i64**> |  | [optional]
**created_on** | **String** |  | 
**updated_on** | **String** |  | 
**state** | **State** |  (enum: RESOLVED, UNRESOLVED) | 
**content** | **serde_json::Value** |  | 
**creator** | [**models::ApiAccount**](Account.md) |  | 
**pending** | Option<**bool**> |  | [optional]
**resolved_on** | Option<**String**> | The ISO8601 timestamp for when the task was resolved. | [optional]
**resolved_by** | Option<[**models::ApiAccount**](Account.md)> |  | [optional]
**links** | Option<**serde_json::Value**> |  | [optional]
**comment** | Option<[**models::ApiComment**](Comment.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


