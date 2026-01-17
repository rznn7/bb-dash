# ApiPipeline

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**uuid** | Option<**String**> | The UUID identifying the pipeline. | [optional]
**build_number** | Option<**i32**> | The build number of the pipeline. | [optional]
**creator** | Option<[**models::ApiAccount**](Account.md)> |  | [optional]
**repository** | Option<[**models::ApiRepository**](Repository.md)> |  | [optional]
**target** | Option<[**models::ApiPipelineTarget**](PipelineTarget.md)> |  | [optional]
**trigger** | Option<[**models::ApiPipelineTrigger**](PipelineTrigger.md)> |  | [optional]
**state** | Option<[**models::ApiPipelineState**](PipelineState.md)> |  | [optional]
**variables** | Option<[**Vec<models::ApiPipelineVariable>**](PipelineVariable.md)> | The variables for the pipeline. | [optional]
**created_on** | Option<**String**> | The timestamp when the pipeline was created. | [optional]
**completed_on** | Option<**String**> | The timestamp when the Pipeline was completed. This is not set if the pipeline is still in progress. | [optional]
**build_seconds_used** | Option<**i32**> | The number of build seconds used by this pipeline. | [optional]
**configuration_sources** | Option<[**Vec<models::ApiPipelineConfigurationSource>**](PipelineConfigurationSource.md)> | An ordered list of sources of the pipeline configuration | [optional]
**links** | Option<[**models::ApiPipelinesPipelineLinks**](PipelinesPipelineLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


